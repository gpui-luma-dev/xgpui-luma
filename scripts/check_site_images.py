"""Check local HTML image paths before publishing the documentation site."""

from html.parser import HTMLParser
from pathlib import Path
import sys
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1] / "docs"


class ImageSources(HTMLParser):
    def __init__(self):
        super().__init__()
        self.sources = []

    def handle_starttag(self, tag, attrs):
        if tag == "img":
            source = dict(attrs).get("src")
            if source:
                self.sources.append(source)


def check_images(root):
    errors = []
    pages = sorted(root.rglob("*.html"))
    if not pages:
        return [f"No HTML pages found in {root}"]
    for page in pages:
        parser = ImageSources()
        parser.feed(page.read_text())
        for source in parser.sources:
            url = urlsplit(source)
            if url.scheme or url.netloc:
                continue
            path = unquote(url.path)
            image = root / path.lstrip("/") if path.startswith("/") else page.parent / path
            if not path or not image.is_file() or image.stat().st_size == 0:
                errors.append(f"{page.relative_to(root)}: missing or empty image {source}")
    return errors


if __name__ == "__main__":
    errors = check_images(ROOT)
    if errors:
        print("\n".join(errors), file=sys.stderr)
        sys.exit(1)
    print("All local HTML image paths exist and are non-empty.")
