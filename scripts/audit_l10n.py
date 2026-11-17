"""Repeatable coverage review aid; use luma-l10n-check for authoritative Fluent validation.

Run from the repository root: python3 scripts/audit_l10n.py > /tmp/l10n-audit.json
Equal text is a review candidate, not automatically an untranslated defect.
This script never edits catalogs or infers UI reachability from a string inventory.
"""
import argparse
import json
from collections import defaultdict
from pathlib import Path
import re


def entries(path):
    return {match[1]: match[2].rstrip() for match in re.finditer(
        r'^([a-zA-Z][\w-]*)\s*=\s*([\s\S]*?)(?=^-?[A-Za-z][\w-]*\s*=|^#|\Z)', path.read_text(), re.M)}



def audit(usage_path=None):
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
        'runtime_english_pattern_inference': 'generated::PATTERNS' in Path('apps/xgpui-lab/src/l10n/copy.rs').read_text(),
        'typed_source_reference_count': sum(bool(e.get('explicit')) for e in registry),
        'ui_boundary_candidate_count': sum(bool(e.get('boundaries')) for e in registry),
        'manifest_purpose': 'Resource contract for static lookup and typed IDs; not a UI reachability claim.',
        'translation_review_status': 'Draft; native-speaker review pending for de, ja, ar.',
        'formatted_registry_count': sum(e['parts'] is not None for e in registry),
        'legacy_source_fragment_collisions': [v for _, v in sorted(groups.items()) if len(v) > 1],
        'used_manifest_count': len(used),
        'manifest_ids_missing_in_english': sorted(used-reference.keys()),
        'english_ids_absent_from_manifest': sorted(reference.keys()-used),
        'registry_ids_present_in_manifest': sum(e['id'] in used for e in registry),
        'fluent_select_message_count': sum('->' in value for value in reference.values()),
    })
    if usage_path:
        usage = json.loads(Path(usage_path).read_text())
        rendered = set(usage['ids'])
        report['lab']['render_observation'] = {
            'scope': 'All destinations, native/Parley, narrow light en-XA and wide dark ar-XB; default states only.',
            'rendered_id_count': len(rendered),
            'rendered_ids': sorted(rendered),
            'unexercised_english_ids': sorted(reference.keys()-rendered),
            'unresolved_review_candidates': usage['unresolved'],
            'limitation': 'Dynamic interaction states and technical/data literals require separate review; unknown text is not automatically a translation defect.'}
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--usage', help='JSON captured by LAB_COPY_USAGE during the headless destination test')
    args = parser.parse_args()
    print(json.dumps(audit(args.usage), ensure_ascii=False, indent=2))
