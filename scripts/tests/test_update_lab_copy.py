"""Audit regression: refreshing an inventory must not mutate reviewed Fluent text."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1]/'update_lab_copy.py'


class CatalogUpdate(unittest.TestCase):
    def test_multiline_translation_is_preserved_and_refresh_is_idempotent(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            locales = root/'apps/xgpui-lab/locales'
            message_id = 'ui-'+hashlib.sha256(b'Example').hexdigest()[:12]
            for tag in ('en', 'de', 'ja', 'ar'):
                (locales/tag).mkdir(parents=True)
                (locales/tag/'lab.ftl').write_text(
                    f'kept = Independent message\n{message_id} = First line\n    Second line\n'
                    '    { $count ->\n        [one] One\n       *[other] Other\n}\n'
                )
            (locales/'used-ids.txt').write_text(f'kept\n{message_id}\n')
            inventory = root/'inventory.json'
            inventory.write_text(json.dumps({'Example': {'kind': 'text', 'files': ['example.rs']}}))
            def update():
                subprocess.run([sys.executable, str(SCRIPT), str(inventory)], cwd=root,
                               check=True, capture_output=True)
            update()
            before = {str(p): p.read_bytes() for p in locales.rglob('*') if p.is_file()}
            self.assertIn(f'{message_id} = First line\n    Second line\n    {{ $count ->\n        [one] One\n       *[other] Other\n}}\n', (locales/'de/lab.ftl').read_text())
            update()
            self.assertEqual(before, {str(p): p.read_bytes() for p in locales.rglob('*') if p.is_file()})


    def test_final_manifest_id_without_trailing_newline_is_not_duplicated(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            locales = root/'apps/xgpui-lab/locales'
            message_id = 'ui-'+hashlib.sha256(b'Example').hexdigest()[:12]
            for tag in ('en', 'de', 'ja', 'ar'):
                (locales/tag).mkdir(parents=True)
                (locales/tag/'lab.ftl').write_text(f'kept = Independent message\n{message_id} = Example\n')
            (locales/'used-ids.txt').write_text(f'kept\n\n# Extracted Lab copy\n{message_id}')
            inventory = root/'inventory.json'
            inventory.write_text(json.dumps({'Example': {'kind': 'text', 'files': ['example.rs']}}))
            subprocess.run([sys.executable, str(SCRIPT), str(inventory)], cwd=root, check=True, capture_output=True)
            used = (locales/'used-ids.txt').read_text()
            self.assertEqual(used.splitlines().count(message_id), 1, used)
            self.assertEqual(used, f'kept\n\n# Extracted Lab copy\n{message_id}\n')


if __name__ == '__main__':
    unittest.main()
