"""Exercise GitHub release finalization offline; never contact GitHub."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import textwrap
import unittest


class ReleaseWorkflowTests(unittest.TestCase):
    def finalize(self, state, upload_status=0):
        workflow = (Path(__file__).resolve().parents[1] / ".github/workflows/release.yml").read_text()
        step = workflow.split("      - name: Create GitHub release\n", 1)[1].split("\n      - name:", 1)[0]
        script = textwrap.dedent(step.split("        run: |\n", 1)[1])
        script = script.replace("${{ steps.meta.outputs.framework_version }}", "3.2.0")
        script = script.replace("${{ github.repository }}", "example/luma")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "dist").mkdir()
            mock = root / "gh"
            mock.write_text("#!/usr/bin/env python3\n" + textwrap.dedent('''\
                import json, os, sys
                args = sys.argv[1:]
                with open("calls.jsonl", "a") as log:
                    log.write(json.dumps(args) + "\\n")
                if args[:2] == ["release", "view"]:
                    if os.environ["RELEASE_STATE"] == "missing":
                        sys.exit(1)
                    if "--json" in args:
                        print("true" if os.environ["RELEASE_STATE"] == "draft" else "false")
                if args[:2] == ["release", "upload"]:
                    sys.exit(int(os.environ["UPLOAD_STATUS"]))
            '''))
            mock.chmod(0o755)
            result = subprocess.run(["bash", "-c", script], cwd=root, capture_output=True, text=True, env={
                **os.environ, "PATH": str(root) + os.pathsep + os.environ["PATH"],
                "TAG": "v1.0.2", "VERSION": "1.0.2", "RELEASE_STATE": state,
                "UPLOAD_STATUS": str(upload_status),
            })
            calls = [json.loads(line) for line in (root / "calls.jsonl").read_text().splitlines()]
            return result, calls

    def test_existing_draft_keeps_notes_and_publishes_after_upload(self):
        result, calls = self.finalize("draft")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls[-2][:2], ["release", "upload"])
        self.assertEqual(calls[-1], ["release", "edit", "v1.0.2", "--verify-tag", "--draft=false"])
        self.assertFalse(any("--notes-file" in call or "--generate-notes" in call for call in calls))

    def test_upload_failure_leaves_draft_unpublished(self):
        result, calls = self.finalize("draft", upload_status=1)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(any(call[:2] == ["release", "edit"] for call in calls))

    def test_published_release_only_uploads_assets(self):
        result, calls = self.finalize("published")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls[-1][:2], ["release", "upload"])

    def test_missing_release_uses_generated_fallback_notes(self):
        result, calls = self.finalize("missing")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls[-1][:2], ["release", "create"])
        self.assertIn("--verify-tag", calls[-1])
        self.assertIn("--notes-file", calls[-1])


if __name__ == "__main__":
    unittest.main()
