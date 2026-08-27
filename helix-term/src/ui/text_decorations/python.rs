use helix_core::text_annotations::LineAnnotation;
use helix_core::Position;
use helix_view::Theme;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::{
    python::InlineOutput,
    ui::document::{LinePos, TextRenderer},
};

use super::Decoration;

const MAX_OUTPUT_LINES: usize = 200;
static FINAL_OUTPUT_LINES: Lazy<Mutex<HashSet<(usize, usize)>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

/// Renders the latest result of Python executions as virtual lines below the
/// source line that produced them. It is deliberately a decoration: the
/// output is never inserted into, or saved with, the source document.
pub struct PythonOutput {
    outputs: Vec<InlineOutput>,
    style: helix_view::theme::Style,
    output_style: helix_view::theme::Style,
}

impl PythonOutput {
    pub fn new(outputs: Vec<InlineOutput>, theme: &Theme) -> Self {
        Self {
            outputs,
            style: theme.get("ui.virtual.inlay-hint"),
            output_style: theme.get("ui.virtual.wrap"),
        }
    }

    pub fn annotation(outputs: Vec<InlineOutput>) -> Self {
        Self {
            outputs,
            style: Default::default(),
            output_style: Default::default(),
        }
    }

    fn output_lines(output: &InlineOutput) -> usize {
        1 + output.output.lines().take(MAX_OUTPUT_LINES).count()
    }

    fn title(output: &InlineOutput) -> String {
        let label = output
            .label
            .split_once(':')
            .and_then(|(kind, rest)| rest.rsplit_once(':').map(|(_, lines)| (kind, lines)))
            .map_or_else(
                || output.label.clone(),
                |(kind, lines)| format!("{kind} {lines}"),
            );
        let timing = output
            .elapsed_ms
            .map_or_else(String::new, |ms| format!(", {ms}ms"));
        format!("▾ {label} [{}{}]", output.status, timing)
    }

    fn draw(&self, renderer: &mut TextRenderer, pos: LinePos, virt_off: Position) -> Position {
        let outputs = self
            .outputs
            .iter()
            .filter(|output| {
                output.anchor_line == pos.doc_line
                    && FINAL_OUTPUT_LINES
                        .lock()
                        .expect("Python output state mutex poisoned")
                        .contains(&(output.anchor_line, output.anchor_char))
            })
            .collect::<Vec<_>>();
        if outputs.is_empty() {
            return Position::new(0, 0);
        }

        let mut row = pos.visual_line + virt_off.row as u16;
        let x = renderer.viewport.x.saturating_add(1);
        let width = renderer.viewport.width.saturating_sub(1) as usize;
        let mut rows = 0;

        for output in outputs {
            let header = Self::title(output);
            renderer.set_stringn(x, row, &header, width, self.style);
            row = row.saturating_add(1);
            rows += 1;

            for line in output.output.lines().take(MAX_OUTPUT_LINES) {
                let text = format!("│ {line}");
                renderer.set_stringn(x, row, &text, width, self.output_style);
                row = row.saturating_add(1);
                rows += 1;
            }
        }

        Position::new(rows, 0)
    }
}

impl LineAnnotation for PythonOutput {
    fn insert_virtual_lines(
        &mut self,
        line_end_char_idx: usize,
        _line_end_visual_pos: Position,
        doc_line: usize,
    ) -> Position {
        let mut final_lines = FINAL_OUTPUT_LINES
            .lock()
            .expect("Python output state mutex poisoned");
        for output in self
            .outputs
            .iter()
            .filter(|output| output.anchor_line == doc_line)
        {
            let key = (output.anchor_line, output.anchor_char);
            if output.anchor_char <= line_end_char_idx {
                final_lines.insert(key);
            } else {
                final_lines.remove(&key);
            }
        }
        Position::new(
            self.outputs
                .iter()
                .filter(|output| {
                    output.anchor_line == doc_line && output.anchor_char <= line_end_char_idx
                })
                .map(Self::output_lines)
                .sum(),
            0,
        )
    }
}

impl Decoration for PythonOutput {
    fn render_virt_lines(
        &mut self,
        renderer: &mut TextRenderer,
        pos: LinePos,
        virt_off: Position,
    ) -> Position {
        self.draw(renderer, pos, virt_off)
    }
}
