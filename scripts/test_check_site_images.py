"""Image validation regressions for publishing renamed screenshots."""

from pathlib import Path
import tempfile
import unittest

from check_site_images import check_images


class SiteImagesTests(unittest.TestCase):
    def test_relative_root_and_encoded_paths_ignore_external_images(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "nested").mkdir()
            (root / "shot 1.png").write_bytes(b"image")
            (root / "nested/index.html").write_text(
                '<img src="../shot%201.png?cache=1"><img src="/shot%201.png">'
                '<img src="https://example.com/external.png"><img src="data:image/png;base64,abc">')
            self.assertEqual(check_images(root), [])

    def test_missing_renamed_and_empty_images_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "renamed.png").write_bytes(b"image")
            (root / "empty.png").touch()
            (root / "index.html").write_text('<img src="old-name.png"><img src="empty.png">')
            errors = check_images(root)
            self.assertEqual(len(errors), 2)
            self.assertIn("old-name.png", errors[0])
            self.assertIn("empty.png", errors[1])


if __name__ == "__main__":
    unittest.main()
