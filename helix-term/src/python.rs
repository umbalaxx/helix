//! Python execution support.
//!
//! This module deliberately keeps execution state separate from the edited
//! document.  It is the model used by the eventual inline notebook renderer:
//! a document has cells, and a project session owns their execution state.

use std::{
    collections::{BTreeMap, HashMap},
    io::{BufRead, BufReader, Write},
    ops::Range,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use anyhow::{anyhow, Context as _};
use helix_view::DocumentId;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

#[derive(Serialize)]
struct Request<'a> {
    code: &'a str,
}

#[derive(Deserialize)]
struct Response {
    output: String,
    error: Option<String>,
    interrupted: bool,
}

static SESSIONS: Lazy<Mutex<HashMap<PathBuf, Arc<Mutex<Session>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static SESSION_PIDS: Lazy<Mutex<HashMap<PathBuf, u32>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static SESSION_RUNNING: Lazy<Mutex<HashMap<PathBuf, bool>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static SESSION_INTERRUPTING: Lazy<Mutex<HashMap<PathBuf, bool>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_LABELS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static OUTPUT_BUFFERS: Lazy<Mutex<HashMap<PathBuf, DocumentId>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static OUTPUTS: Lazy<Mutex<HashMap<PathBuf, BTreeMap<String, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn begin_task(label: String) {
    ACTIVE_LABELS
        .lock()
        .expect("Python task-label mutex poisoned")
        .push(label);
    ACTIVE_TASKS.fetch_add(1, Ordering::SeqCst);
}

pub fn end_task(label: &str) -> usize {
    let mut labels = ACTIVE_LABELS
        .lock()
        .expect("Python task-label mutex poisoned");
    if let Some(index) = labels.iter().position(|item| item == label) {
        labels.remove(index);
    }
    ACTIVE_TASKS.fetch_sub(1, Ordering::SeqCst) - 1
}

pub fn active_tasks() -> usize {
    ACTIVE_TASKS.load(Ordering::SeqCst)
}

pub fn active_labels() -> Vec<String> {
    ACTIVE_LABELS
        .lock()
        .expect("Python task-label mutex poisoned")
        .clone()
}

/// Execute code in the persistent IPython session for `project`.
///
/// The first version intentionally uses a small line-delimited protocol. It
/// keeps the process boundary independent from terminal prompts and leaves us
/// room to replace the output scratch buffer with inline notebook decorations.
pub fn execute(project: &Path, code: &str) -> anyhow::Result<String> {
    let mut sessions = SESSIONS.lock().expect("python session mutex poisoned");
    let session = if let Some(session) = sessions.get(project) {
        session.clone()
    } else {
        let session = spawn_session(project)?;
        SESSION_PIDS
            .lock()
            .expect("Python pid mutex poisoned")
            .insert(project.to_path_buf(), session.child.id());
        let session = Arc::new(Mutex::new(session));
        sessions.insert(project.to_path_buf(), session.clone());
        session
    };
    drop(sessions);
    let mut session = session.lock().expect("Python session mutex poisoned");
    SESSION_RUNNING
        .lock()
        .expect("Python running-state mutex poisoned")
        .insert(project.to_path_buf(), true);
    SESSION_INTERRUPTING
        .lock()
        .expect("Python interrupt-state mutex poisoned")
        .insert(project.to_path_buf(), false);

    let result = (|| {
        let request = serde_json::to_string(&Request { code })?;
        writeln!(session.stdin, "{request}")?;
        session.stdin.flush()?;

        let mut line = String::new();
        session
            .stdout
            .read_line(&mut line)
            .context("Python session exited without returning a result")?;
        let response: Response = serde_json::from_str(&line)?;
        if response.interrupted {
            return Ok("[Python execution interrupted]\n".to_owned());
        }
        if let Some(error) = response.error {
            return Err(anyhow!("{error}\n{}", response.output));
        }
        Ok(response.output)
    })();
    SESSION_RUNNING
        .lock()
        .expect("Python running-state mutex poisoned")
        .insert(project.to_path_buf(), false);
    SESSION_INTERRUPTING
        .lock()
        .expect("Python interrupt-state mutex poisoned")
        .insert(project.to_path_buf(), false);
    result
}

pub fn sessions() -> Vec<PathBuf> {
    SESSIONS
        .lock()
        .expect("python session mutex poisoned")
        .keys()
        .cloned()
        .collect()
}

pub fn stop_all() {
    let sessions = SESSIONS
        .lock()
        .expect("python session mutex poisoned")
        .drain()
        .map(|(_, session)| session)
        .collect::<Vec<_>>();
    for session in sessions {
        if let Ok(mut session) = session.try_lock() {
            let _ = session.child.kill();
        }
    }
    SESSION_PIDS
        .lock()
        .expect("Python pid mutex poisoned")
        .clear();
    SESSION_RUNNING
        .lock()
        .expect("Python running-state mutex poisoned")
        .clear();
    SESSION_INTERRUPTING
        .lock()
        .expect("Python interrupt-state mutex poisoned")
        .clear();
}

pub fn interrupt(project: &Path) -> anyhow::Result<()> {
    let running = SESSION_RUNNING
        .lock()
        .expect("Python running-state mutex poisoned")
        .get(project)
        .copied()
        .unwrap_or(false);
    if !running {
        return Err(anyhow!("no Python execution is currently running"));
    }
    let mut interrupting = SESSION_INTERRUPTING
        .lock()
        .expect("Python interrupt-state mutex poisoned");
    if interrupting.get(project).copied().unwrap_or(false) {
        return Ok(());
    }
    interrupting.insert(project.to_path_buf(), true);
    drop(interrupting);
    let pid = SESSION_PIDS
        .lock()
        .expect("Python pid mutex poisoned")
        .get(project)
        .copied()
        .ok_or_else(|| anyhow!("no active Python session for {}", project.display()))?;

    #[cfg(unix)]
    {
        // The helper is the direct child, so SIGINT is equivalent to pressing
        // Ctrl-C in its IPython terminal while keeping the kernel alive.
        let result = unsafe { libc::kill(pid as libc::pid_t, libc::SIGINT) };
        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        Err(anyhow!(
            "interrupting Python is not supported on this platform yet"
        ))
    }
}

pub fn output_buffer(project: &Path) -> Option<DocumentId> {
    OUTPUT_BUFFERS
        .lock()
        .expect("Python output mutex poisoned")
        .get(project)
        .copied()
}

pub fn set_output_buffer(project: &Path, id: DocumentId) {
    OUTPUT_BUFFERS
        .lock()
        .expect("Python output mutex poisoned")
        .insert(project.to_path_buf(), id);
}

/// Replace the latest output associated with `label` and render all outputs
/// for this project as a notebook-style inspection buffer.
pub fn update_output(project: &Path, label: String, output: String) -> String {
    let mut outputs = OUTPUTS.lock().expect("Python output mutex poisoned");
    outputs
        .entry(project.to_path_buf())
        .or_default()
        .insert(label, output);

    let mut rendered = format!("Python output: {}\n\n", project.display());
    for (label, output) in outputs.get(project).expect("output was just inserted") {
        rendered.push_str(&format!("── {label} ──\n"));
        rendered.push_str(output);
        if !output.ends_with('\n') {
            rendered.push('\n');
        }
        rendered.push('\n');
    }
    rendered
}

fn spawn_session(project: &Path) -> anyhow::Result<Session> {
    let helper = helix_loader::runtime_file("python/helix_session.py");
    let mut child = Command::new("uv")
        .args(["run", "--project"])
        .arg(project)
        // Keep the project environment authoritative, but make the optional
        // interactive dependency available even when the project itself does
        // not list IPython yet.
        .args(["--with", "ipython", "python", "-u"])
        .arg(helper)
        .current_dir(project)
        .env("PYTHONPATH", project)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("could not start `uv`; is uv installed?")?;
    let stdin = child
        .stdin
        .take()
        .context("Python session stdin unavailable")?;
    let stdout = child
        .stdout
        .take()
        .context("Python session stdout unavailable")?;
    Ok(Session {
        child,
        stdin,
        stdout: BufReader::new(stdout),
    })
}

/// Find the project root used by uv, starting at a Python file's directory.
pub fn project_root(file: Option<&Path>, cwd: &Path) -> Option<PathBuf> {
    let start = file.and_then(Path::parent).unwrap_or(cwd);
    start
        .ancestors()
        .find(|path| path.join("pyproject.toml").is_file())
        .map(Path::to_path_buf)
        .or_else(|| Some(cwd.to_path_buf()))
}

/// A Python cell delimited by a `# %%` marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// Zero-based line range, with the end excluded.
    pub lines: Range<usize>,
}

/// Return the cell containing `line` (or the nearest cell after it).
pub fn cell_at(text: &str, line: usize) -> Cell {
    let lines: Vec<&str> = text.lines().collect();
    let markers: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, value)| is_cell_marker(value).then_some(index))
        .collect();

    let start = markers
        .iter()
        .copied()
        .rev()
        .find(|marker| *marker <= line)
        .unwrap_or(0);
    let end = markers
        .iter()
        .copied()
        .find(|marker| *marker > start)
        .unwrap_or(lines.len());

    Cell { lines: start..end }
}

fn is_cell_marker(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed == "#%%" || trimmed.starts_with("# %%")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_hash_percent_cells() {
        let text = "# %%\na = 1\n# %% [markdown]\ntext\n#%%\nb = 2\n";
        assert_eq!(cell_at(text, 1).lines, 0..2);
        assert_eq!(cell_at(text, 3).lines, 2..4);
        assert_eq!(cell_at(text, 5).lines, 4..6);
    }

    #[test]
    fn code_before_first_marker_is_first_cell() {
        assert_eq!(cell_at("import os\n# %%\nx = 1\n", 0).lines, 0..1);
    }
}
