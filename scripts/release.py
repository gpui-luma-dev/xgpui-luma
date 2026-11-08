"""Resumable crates.io release; never prints or persists registry credentials."""

import argparse
import io
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import tarfile
import time
import urllib.error
import urllib.request

REPORT = Path('target/release-report/status.json')
USER_AGENT = 'xgpui-luma-release (https://gpui-luma.dev)'


class ReleaseError(Exception):
    pass


def download(url, attempts=4, missing_ok=False):
    for attempt in range(attempts):
        try:
            request = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
            with urllib.request.urlopen(request, timeout=30) as response:
                return response.read()
        except urllib.error.HTTPError as error:
            if error.code == 404 and missing_ok:
                return None
            if error.code not in (429, 500, 502, 503, 504):
                raise ReleaseError(f'Registry lookup failed: HTTP {error.code} at {url}') from error
        except (urllib.error.URLError, TimeoutError):
            pass
        if attempt + 1 < attempts:
            time.sleep(min(2 ** attempt, 10))
    raise ReleaseError(f'Registry lookup unavailable after {attempts} attempts: {url}')


def save(report):
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    temporary = REPORT.with_suffix('.tmp')
    temporary.write_text(json.dumps(report, indent=2) + '\n')
    temporary.replace(REPORT)


def discover():
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'], text=True))
    packages = {package['name']: package for package in metadata['packages']}
    names = os.environ['LIB_PACKAGES'].split()
    version = os.environ['VERSION']
    for name in names:
        package = packages[name]
        if package['version'] != version or package['publish'] != ['crates-io']:
            raise ReleaseError(f'{name}: expected version {version} and publish = ["crates-io"]')
    # LIB_PACKAGES must put local dependencies before their dependents.
    for index, name in enumerate(names):
        for dependency in packages[name]['dependencies']:
            if dependency['kind'] != 'dev' and dependency['name'] in names[index:]:
                raise ReleaseError(f'{name}: dependency {dependency["name"]} must publish first')
    return {'version': version, 'commit': os.environ['GITHUB_SHA'], 'phase': 'preflight',
            'error': None, 'packages': {name: {'status': 'not checked'} for name in names}}


def preflight(report, publish, allow_new, require_unpublished=False):
    version = report['version']
    new = []
    published = []
    for name, state in report['packages'].items():
        exists = download(f'https://crates.io/api/v1/crates/{name}', missing_ok=True) is not None
        released = download(f'https://crates.io/api/v1/crates/{name}/{version}', missing_ok=True) is not None
        state.update(new_crate=not exists, status='already published; verification pending' if released else 'pending')
        if not exists:
            new.append(name)
        if released:
            published.append(name)
        if require_unpublished:
            print(f'{name} {version}: {"already published" if released else "not published"}', flush=True)
        save(report)
    if require_unpublished and published:
        raise ReleaseError(f'Version {version} is already published on crates.io for: ' + ', '.join(published) +
                           '. Bump workspace.package.version and matching internal dependency versions in Cargo.toml, '
                           'update Cargo.lock, then rerun just preflight-release. No crates uploaded.')
    pending = any(state['status'] == 'pending' for state in report['packages'].values())
    if publish and pending and not os.environ.get('CARGO_REGISTRY_TOKEN'):
        raise ReleaseError('Missing CARGO_REGISTRY_TOKEN repository secret; no crates uploaded.')
    if publish and new and not allow_new:
        raise ReleaseError('New crate registrations: ' + ', '.join(new) +
                           '. No crates uploaded. Configure token permission to create these crates, '
                           'then run with allow_new_crates enabled. This input does not grant token permissions.')
    if new:
        print('New crates need creation permission and matching token name scope: ' + ', '.join(new), flush=True)
    print('Registry preflight complete. Token publish permissions can only be confirmed by the registry upload.', flush=True)


def contents(data):
    with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
        return {member.name: archive.extractfile(member).read() for member in archive if member.isfile()}


def local_contents(name, report):
    filename = f'{name}-{report["version"]}.crate'
    for directory in (Path('target/package'), Path('target/package/tmp-registry')):
        path = directory / filename
        if path.is_file():
            expected = contents(path.read_bytes())
            vcs = json.loads(expected[f'{name}-{report["version"]}/.cargo_vcs_info.json'])['git']
            if vcs['sha1'] != report['commit'] or vcs.get('dirty', False):
                raise ReleaseError(f'{name}: archive must come from the clean release commit')
            return expected
    raise ReleaseError(f'{name}: missing verified package archive')


def verify(name, report, expected, wait=False):
    url = f'https://static.crates.io/crates/{name}/{name}-{report["version"]}.crate'
    for attempt in range(12 if wait else 1):
        # Missing CDN objects can return 403. Only request an archive once the
        # registry API confirms that this version exists.
        version_url = f'https://crates.io/api/v1/crates/{name}/{report["version"]}'
        released = download(version_url, missing_ok=True) is not None
        data = download(url) if released else None
        if data is not None:
            if contents(data) != expected:
                raise ReleaseError(f'{name}: published contents differ. Do not overwrite or retry with changed '
                                   'sources; use the original commit or a new version.')
            return True
        if wait and attempt < 11:
            time.sleep(10)
    return False


def publish_packages(report, publish, allow_new):
    # Recheck registry state and all archives before the first irreversible upload.
    preflight(report, publish, allow_new)
    expected = {name: local_contents(name, report) for name in report['packages']}
    for name, state in report['packages'].items():
        if state['status'].startswith('already published'):
            if not verify(name, report, expected[name], wait=True):
                raise ReleaseError(f'{name}: registry version exists but archive is unavailable; retry same commit later')
            state['status'] = 'verified'
        save(report)
    if not publish:
        report['phase'] = 'verified only; no uploads, tag, or release'
        save(report)
        return
    report['phase'] = 'publishing'
    for name, state in report['packages'].items():
        if state['status'] == 'verified':
            print(f'Skipping verified {name}', flush=True)
            continue
        state['status'] = 'uploading; registry outcome not yet verified'
        save(report)
        print(f'Publishing {name} {report["version"]}', flush=True)
        command = ['cargo', 'publish', '--locked', '--registry', 'crates-io', '--config',
                   'registries.crates-io.credential-provider="cargo:token"', '--package', name]
        try:
            result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                    text=True, timeout=600)
        except subprocess.TimeoutExpired as error:
            output = error.stdout or b''
            if isinstance(output, bytes):
                output = output.decode(errors='replace')
            result = subprocess.CompletedProcess(command, 124, output + '\nCargo publish timed out.\n')
        token = os.environ.get('CARGO_REGISTRY_TOKEN', '')
        output = result.stdout.replace(token, '[REDACTED]') if token else result.stdout
        (REPORT.parent / f'{name}.log').write_text(output)
        print(output, end='', flush=True)
        permission_failure = re.search(r'\b(?:401|403)\b', output) is not None
        if result.returncode and permission_failure:
            state['status'] = 'permission denied'
            save(report)
            raise ReleaseError(f'{name}: token rejected. Check expiry, crate-name scope, '
                               'new-crate creation permission and account ownership. Update '
                               'CARGO_REGISTRY_TOKEN, then re-run this same commit; verified crates are skipped.')
        # A failed Cargo command may already have uploaded successfully (index timeout).
        if verify(name, report, expected[name], wait=True):
            state['status'] = 'verified'
            save(report)
            continue
        state['status'] = 'upload failed or not yet visible; retry same commit'
        save(report)
        raise ReleaseError(f'{name}: upload not verified (Cargo exit {result.returncode}). '
                           'Inspect saved log and re-run the same commit. No automatic upload retry was attempted.')
    report['phase'] = 'all crates verified; ready to tag and release'
    save(report)


def summary():
    if not REPORT.exists():
        text = 'Release stopped before registry preflight. Check the failed workflow step.\n'
    else:
        report = json.loads(REPORT.read_text())
        text = f'## crates.io {report["version"]}\n\nCommit: `{report["commit"]}`\n\n{report["phase"]}\n\n'
        text += '| Crate | Status |\n| --- | --- |\n'
        for name, state in report['packages'].items():
            text += f'| {name} | {state["status"]} |\n'
        if report['error']:
            text += f'\n**Failure:** {report["error"]}\n'
        text += '\nFor a partial release, re-run the original commit after fixing the failure. '
        text += 'Published versions are immutable; do not edit sources and retry the same version.\n'
    print(text)
    if os.environ.get('GITHUB_STEP_SUMMARY'):
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as file:
            file.write(text)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('phase', choices=['preflight', 'publish', 'summary'])
    parser.add_argument('--require-unpublished', action='store_true',
                        help='Reject an existing crates.io version when checking a new release candidate')
    args = parser.parse_args()
    if args.require_unpublished and args.phase != 'preflight':
        parser.error('--require-unpublished is only valid with preflight')
    if args.phase == 'summary':
        summary()
        return 0
    report = None
    try:
        report = discover()
        if args.phase == 'publish' and REPORT.exists():
            previous = json.loads(REPORT.read_text())
            if (previous['commit'], previous['version']) != (report['commit'], report['version']):
                raise ReleaseError('Release report belongs to a different commit or version')
        save(report)
        publish = os.environ.get('PUBLISH_RELEASE') == 'true'
        allow_new = os.environ.get('ALLOW_NEW_CRATES') == 'true'
        if args.phase == 'preflight':
            preflight(report, publish, allow_new, require_unpublished=args.require_unpublished)
        else:
            publish_packages(report, publish, allow_new)
        return 0
    except (ReleaseError, OSError, ValueError, KeyError, subprocess.SubprocessError, tarfile.TarError) as error:
        message = str(error)
        token = os.environ.get('CARGO_REGISTRY_TOKEN', '')
        if token:
            message = message.replace(token, '[REDACTED]')
        if report is not None:
            report.update(phase='failed', error=message)
            save(report)
        print(f'::error::{message.replace(chr(10), " ")}', flush=True)
        return 1


if __name__ == '__main__':
    sys.exit(main())
