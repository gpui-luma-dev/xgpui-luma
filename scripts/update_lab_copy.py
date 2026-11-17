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
for source, meta in sorted(json.loads(Path(sys.argv[1]).read_text()).items()):
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
                        value=value, parts=parts, files=sorted(set(meta['files']))))
if '--check' in sys.argv:
    if json.loads((root/'copy.json').read_text()) != entries:
        raise SystemExit('Lab catalog inventory changed; run update_lab_copy.py and review translations')
    print(f'{len(entries)} catalog entries verified')
    raise SystemExit(0)
(root/'copy.json').write_text(json.dumps(entries, ensure_ascii=False, indent=2)+'\n')
for tag in ('en', 'de', 'ja', 'ar'):
    path = root/tag/'lab.ftl'
    source = path.read_text()
    translations = dict(re.findall(r'^(ui-[a-f0-9]+) = (.*(?:\n[ \t]+.*)*)', source, re.M))
    source = re.sub(r'^ui-[a-f0-9]+ = .*\n(?:[ \t]+.*\n)*', '', source, flags=re.M)
    source = source.replace('# Extracted Lab copy (stable generated IDs; source references in copy.json).\n', '')
    source = source.rstrip()+'\n\n# Extracted Lab copy (stable generated IDs; source references in copy.json).\n'
    source += '\n'.join(e['id']+' = '+(e['value'] if tag == 'en' else translations.get(e['id'],e['value'])).replace('\n','\n    ') for e in entries)+'\n'
    path.write_text(source)
p = root/'used-ids.txt'
s = re.sub(r'^ui-[a-f0-9]+\n', '', p.read_text(), flags=re.M).replace('# Extracted Lab copy\n','')
p.write_text(s.rstrip()+'\n\n# Extracted Lab copy\n'+'\n'.join(e['id'] for e in entries)+'\n')
print(f'{len(entries)} catalog entries')
