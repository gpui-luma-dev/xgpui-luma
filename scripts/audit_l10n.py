"""Repeatable coverage review aid; use luma-l10n-check for authoritative Fluent validation.

Run from the repository root: python3 scripts/audit_l10n.py > /tmp/l10n-audit.json
Equal text is a review candidate, not automatically an untranslated defect.
This script never edits catalogs or infers UI reachability from a string inventory.
"""
import json
from collections import defaultdict
from pathlib import Path
import re


def entries(path):
    result = {}
    current = None
    for line in path.read_text().splitlines():
        match = re.match(r'^([a-zA-Z][\w-]*)\s*=\s*(.*)$', line)
        if match:
            current = match[1]
            result[current] = match[2]
        elif line.startswith((' ', '\t')) and current:
            result[current] += '\n' + line
        else:
            current = None
    return result


def audit():
    roots = {'lab': Path('apps/xgpui-lab/locales'), 'sdk': Path('crates/l10n/locales')}
    report = {'note': 'Text equality and registry membership do not establish translation quality or UI coverage.'}
    for name, root in roots.items():
        filename = 'lab.ftl' if name == 'lab' else 'sdk.ftl'
        reference = entries(root/'en'/filename)
        locales = {}
        for tag in ('de', 'ja', 'ar'):
            values = entries(root/tag/filename)
            locales[tag] = {
                'message_count': len(values),
                'missing_ids': sorted(reference.keys()-values.keys()),
                'unknown_ids': sorted(values.keys()-reference.keys()),
                'identical_to_english_ids': sorted(k for k in reference.keys() & values.keys() if reference[k] == values[k]),
            }
        report[name] = {'reference_message_count': len(reference), 'locales': locales}
    registry = json.loads((roots['lab']/'copy.json').read_text())
    groups = defaultdict(list)
    for entry in registry:
        if entry['parts'] is not None:
            groups[tuple(entry['parts'])].append({'id': entry['id'], 'source': entry['source']})
    used = {line.strip() for line in (roots['lab']/'used-ids.txt').read_text().splitlines()
            if line.strip() and not line.startswith('#')}
    reference = entries(roots['lab']/'en/lab.ftl')
    report['lab'].update({
        'registry_count': len(registry),
        'formatted_registry_count': sum(e['parts'] is not None for e in registry),
        'duplicate_capture_groups': [v for _, v in sorted(groups.items()) if len(v) > 1],
        'used_manifest_count': len(used),
        'manifest_ids_missing_in_english': sorted(used-reference.keys()),
        'english_ids_absent_from_manifest': sorted(reference.keys()-used),
        'registry_ids_present_in_manifest': sum(e['id'] in used for e in registry),
        'fluent_select_message_count': sum('->' in value for value in reference.values()),
    })
    return report


if __name__ == '__main__':
    print(json.dumps(audit(), ensure_ascii=False, indent=2))
