"""Mechanical translation lint for the Lab and SDK Fluent catalogs (#265).

Run from the repository root: python3 scripts/lint_translations.py
It catches damage that parity checks cannot see: runaway repetition, dropped
symbols or numbers, mangled sample text and code, and translated entries that
must stay verbatim. It does not judge wording; native review is still pending.
A flagged entry is fixed by hand or set back to its English text.
"""
import re
import sys
from pathlib import Path

CATALOGS = (
    (Path('apps/xgpui-lab/locales'), 'lab.ftl', Path('apps/xgpui-lab/locales/verbatim.txt')),
    (Path('crates/l10n/locales'), 'sdk.ftl', None),
)
LOCALES = ('de', 'ja', 'ar')
# Symbols whose loss changes meaning ("Zoom +" vs "Zoom −", "3×4", "−2–2 EV").
SYMBOLS = '+−–×%…±°→←↑↓≤≥=·'
DIGITS = str.maketrans('٠١٢٣٤٥٦٧٨٩۰۱۲۳۴۵۶۷۸۹０１２３４５６７８９', '0123456789' * 3)
EQUIVALENT = {'...': '…', '〜': '–', '～': '–', '・': '·', '･': '·', '٪': '%', '％': '%', '＋': '+', '＝': '='}
NUMBER = re.compile(r'\d+(?:[.,  ]\d+)*')
# Sample text in another script, emoji, and code must survive translation.
FOREIGN = re.compile(r'[Ͱ-ϿЀ-ӿ֐-׿؀-ۿݐ-ݿࢠ-ࣿ'
                     r'ऀ-෿฀-๿぀-ヿ㐀-鿿가-힯\U0001f300-\U0001faff]')
CODE = re.compile(r'[\w.-]*(?:/src/|::|\w_\w|\w\(\))[\w./:()-]*')


def entries(path):
    """Single-pattern messages by ID; selector messages are left to luma-l10n-check."""
    found = {}
    for match in re.finditer(r'^([a-zA-Z][\w-]*)\s*=\s*([\s\S]*?)(?=^-?[A-Za-z][\w-]*\s*=|^#|\Z)',
                             path.read_text(), re.M):
        value = match[2].rstrip()
        if '->' not in value:
            found[match[1]] = re.sub(r'\n\s+', '\n', value)
    return found


def normalize(text):
    text = re.sub(r'\{ *\$\w+ *\}', '{}', text.translate(DIGITS))
    for old, new in EQUIVALENT.items():
        text = text.replace(old, new)
    return text


def numbers(text):
    return [re.sub(r'\D', '', number) for number in NUMBER.findall(text)]


def repeated(text):
    return re.search(r'(.)\1{7,}', text) or re.search(r'(\S+)(?:\s*\1){3,}', text)


def problems(english, translated, verbatim=False):
    """Reasons a translation is mechanically wrong; empty when it passes."""
    if translated == english:
        return []
    if verbatim:
        return ['must stay verbatim']
    source, text = normalize(english), normalize(translated)
    found = []
    if len(text) > 3 * len(source) + 20 or repeated(text) and not repeated(source):
        found.append('runaway repetition')
    lost = ''.join(symbol for symbol in SYMBOLS if source.count(symbol) > text.count(symbol))
    if lost:
        found.append(f'dropped {lost}')
    expected, actual = numbers(source), numbers(text)
    missing = [n for n in set(expected) if expected.count(n) > actual.count(n)]
    # Spelled-out small counts may become digits ("two ways" → "2つの方法").
    extra = [n for n in set(actual) if actual.count(n) > expected.count(n) and int(n) > 100]
    if missing or extra:
        found.append(f'numbers changed {sorted(missing)} -> {sorted(extra)}')
    kept = [token.strip('.,;:!?') for token in source.split() if FOREIGN.search(token) or CODE.fullmatch(token)]
    altered = [token for token in kept if token not in text]
    if altered:
        found.append(f'sample text or code altered: {" ".join(altered)[:60]}')
    return found


def lint(english, translation, verbatim=()):
    return [(key, reason) for key, value in english.items() if key in translation
            for reason in problems(value, translation[key], value in verbatim)]


def read_verbatim(path):
    if path is None:
        return set()
    return {line for line in path.read_text().splitlines() if line and not line.startswith('#')}


def main():
    failures = 0
    for root, filename, verbatim_path in CATALOGS:
        english = entries(root/'en'/filename)
        verbatim = read_verbatim(verbatim_path)
        unknown = verbatim - set(english.values())
        for source in sorted(unknown):
            print(f'{verbatim_path}: no English message has the text {source!r}')
            failures += 1
        for tag in LOCALES:
            for key, reason in lint(english, entries(root/tag/filename), verbatim):
                print(f'{root/tag/filename}: {key}: {reason}')
                failures += 1
    if failures:
        raise SystemExit(f'{failures} translation problems; fix them or restore the English text')
    print('translations passed the mechanical lint')


if __name__ == '__main__':
    sys.exit(main())
