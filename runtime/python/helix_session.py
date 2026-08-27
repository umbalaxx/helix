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
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            # IPython's displayhook renders the final expression. Do not
            # render result.result again below, or expressions appear twice.
            result = shell.run_cell(request["code"], store_history=True)

        output = stdout.getvalue() + stderr.getvalue()
        error = None
        if result.error_before_exec is not None:
            error = str(result.error_before_exec)
        elif result.error_in_exec is not None:
            error = traceback.format_exc()
        print(json.dumps({"output": output, "error": error}), flush=True)
    except Exception as error:  # Keep the protocol alive after bad requests.
        print(json.dumps({"output": "", "error": repr(error)}), flush=True)
