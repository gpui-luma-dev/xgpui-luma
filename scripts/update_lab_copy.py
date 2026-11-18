"""Merge the Rust literal inventory into the Lab catalog without replacing translations.

Run from the repository root after `cargo run -p lab-copy-catalog -- /tmp/lab-copy.json`:
    python3 scripts/update_lab_copy.py /tmp/lab-copy.json
New translation entries deliberately start in English and require translation review.
"""
import hashlib
import json
from pathlib import Path
import re
import string
import sys

root = Path('apps/xgpui-lab/locales')
entries = []
previous = {entry['source']: entry for entry in json.loads((root/'copy.json').read_text())} if (root/'copy.json').exists() else {}
for source, meta in sorted(json.loads(Path(sys.argv[1]).read_text()).items()):
    if meta.get('explicit') and source in previous:
        record = dict(previous[source])
        record['files'] = sorted(set(meta['files']))
        record['explicit'] = True
        if meta.get('boundaries'):
            record['boundaries'] = sorted(set(meta['boundaries']))
        else:
            record.pop('boundaries', None)
        entries.append(record)
        continue
    parts = None
    value = source
    if meta['kind'] == 'format':
        parts = ['']
        try:
            for literal, field, _, _ in string.Formatter().parse(source):
                parts[-1] += literal
                if field is not None:
                    parts.append('')
        except ValueError:
            continue
        if len(parts) < 2 or any(not part for part in parts[1:-1]):
            continue
        escaped = [''.join({'{': '{ "{" }', '}': '{ "}" }'}.get(c, c) for c in part) for part in parts]
        value = ''.join(part + ('{ $a' + str(i) + ' }' if i < len(parts)-1 else '') for i, part in enumerate(escaped))
    entries.append(dict(id='ui-' + hashlib.sha256(source.encode()).hexdigest()[:12], source=source,
                        value=value, parts=parts, files=sorted(set(meta['files'])),
                        **({'explicit': True} if meta.get('explicit') else {}),
                        **({'boundaries': sorted(set(meta['boundaries']))} if meta.get('boundaries') else {})))
if '--check' in sys.argv:
    if json.loads((root/'copy.json').read_text()) != entries:
        raise SystemExit('Lab catalog inventory changed; run update_lab_copy.py and review translations')
    print(f'{len(entries)} catalog entries verified')
    raise SystemExit(0)
(root/'copy.json').write_text(json.dumps(entries, ensure_ascii=False, indent=2)+'\n')
for tag in ('en', 'de', 'ja', 'ar'):
    path = root/tag/'lab.ftl'
    source = path.read_text()
    translations = {match[1]: match[2].rstrip('\n') for match in re.finditer(r'^(ui-[a-f0-9]+) = ([\s\S]*?)(?=^-?[A-Za-z][\w-]*\s*=|^#|\Z)', source, re.M)}
    source = re.sub(r'^ui-[a-f0-9]+ = [\s\S]*?(?=^-?[A-Za-z][\w-]*\s*=|^#|\Z)', '', source, flags=re.M)
    source = source.replace('# Extracted Lab copy (stable generated IDs; source references in copy.json).\n', '')
    source = source.rstrip()+'\n\n# Extracted Lab copy (stable generated IDs; source references in copy.json).\n'
    def message(entry):
        # Existing Fluent continuation indentation is already significant. Do
        # not add another four spaces every time the inventory is refreshed.
        value = translations.get(entry['id']) if tag != 'en' else None
        if value is None:
            value = entry['value'].replace('\n', '\n    ')
        return entry['id'] + ' = ' + value
    source += '\n'.join(message(e) for e in entries)+'\n'
    path.write_text(source)
p = root/'used-ids.txt'
# The final ID may lack a trailing newline; remove it too rather than duplicate it.
s = re.sub(r'^ui-[a-f0-9]+(?:\n|\Z)', '', p.read_text(), flags=re.M)
s = re.sub(r'^# Extracted Lab copy(?:\n|\Z)', '', s, flags=re.M)
p.write_text(s.rstrip()+'\n\n# Extracted Lab copy\n'+'\n'.join(e['id'] for e in entries)+'\n')
print(f'{len(entries)} catalog entries')
