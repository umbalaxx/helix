"""Small structured bridge between Helix and a persistent IPython shell."""

import contextlib
import io
import json
import sys
import traceback

from IPython.core.interactiveshell import InteractiveShell


shell = InteractiveShell.instance()

for line in sys.stdin:
    try:
        request = json.loads(line)
        stdout = io.StringIO()
        stderr = io.StringIO()
        try:
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                # IPython's displayhook renders the final expression. Do not
                # render result.result again below, or expressions appear twice.
                result = shell.run_cell(request["code"], store_history=True)
        except KeyboardInterrupt:
            print(
                json.dumps(
                    {"output": "[Python execution interrupted]\\n", "error": None, "interrupted": True}
                ),
                flush=True,
            )
            continue

        output = stdout.getvalue() + stderr.getvalue()
        error = None
        interrupted = isinstance(result.error_in_exec, KeyboardInterrupt)
        if interrupted:
            error = None
            output = "[Python execution interrupted]\n"
        elif result.error_before_exec is not None:
            error = str(result.error_before_exec)
        elif result.error_in_exec is not None:
            error = traceback.format_exc()
        print(json.dumps({"output": output, "error": error, "interrupted": interrupted}), flush=True)
    except Exception as error:  # Keep the protocol alive after bad requests.
        print(json.dumps({"output": "", "error": repr(error), "interrupted": False}), flush=True)
