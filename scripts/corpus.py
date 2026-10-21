#!/usr/bin/env python3
"""Offline validation corpus and explicit pinned-source preparation (Python 3.11+)."""
import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NATIVE_COMMAND = ['-m', 'unittest', 'discover', '-s', 'tests', '-v']
REQUIRED = {'REF-PY-PASS': 'pass', 'REF-PY-FAIL': 'fail'}


def fields(value, required, optional=()):
    if not isinstance(value, dict) or set(value) - set(required) - set(optional) or set(required) - set(value):
        raise ValueError(f'invalid fields; required={sorted(required)}, optional={sorted(optional)}: {value}')


def local_path(value):
    if not isinstance(value, str) or Path(value).is_absolute():
        raise ValueError(f'invalid relative path: {value}')
    path = (ROOT / value).resolve()
    if not path.is_relative_to(ROOT) or not path.exists():
        raise ValueError(f'missing or escaping input: {value}')
    return path


def text(value):
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f'expected nonempty text: {value}')


def load_catalog(path=ROOT / 'tests/cases.toml'):
    data = tomllib.loads(path.read_text())
    fields(data, {'version', 'case'})
    if type(data['version']) is not int or data['version'] != 1 or not isinstance(data['case'], list) or not data['case']:
        raise ValueError('unsupported catalog version or empty case list')
    seen = set()
    for case in data['case']:
        fields(case, {'id', 'status', 'kind', 'input', 'expectation'}, {'mode'})
        for value in case.values():
            text(value)
        if case['id'] in seen:
            raise ValueError(f'duplicate case: {case["id"]}')
        seen.add(case['id'])
        if not re.fullmatch(r'[A-Z][A-Z0-9-]+', case['id']):
            raise ValueError('invalid case ID')
        if case['status'] not in {'planned', 'active', 'required'} or case['kind'] not in {'native-python', 'inspection', 'execution'}:
            raise ValueError('invalid case status or kind')
        local_path(case['input'])
        if case['kind'] == 'native-python':
            if case.get('mode') not in {'pass', 'fail'}:
                raise ValueError('native-python requires pass/fail mode')
        elif 'mode' in case:
            raise ValueError('mode is only valid for native-python')
    return data['case']


def load_sources(path=ROOT / 'tests/ecosystem/cases.toml'):
    data = tomllib.loads(path.read_text())
    fields(data, {'version', 'repository'})
    if type(data['version']) is not int or data['version'] != 1 or not isinstance(data['repository'], list) or not data['repository']:
        raise ValueError('unsupported source catalog version or empty repositories')
    seen = set()
    for source in data['repository']:
        fields(source, {'name', 'url', 'commit', 'workflow', 'purpose', 'file'})
        for key in ['name', 'url', 'commit', 'workflow', 'purpose']:
            text(source[key])
        if not re.fullmatch(r'[a-z][a-z0-9-]*', source['name']) or source['name'] in seen:
            raise ValueError('invalid or duplicate repository name')
        seen.add(source['name'])
        if not re.fullmatch(r'https://github.com/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\.git', source['url']) or not re.fullmatch(r'[0-9a-f]{40}', source['commit']):
            raise ValueError('repository requires HTTPS GitHub URL and full commit SHA')
        if not isinstance(source['file'], list) or not source['file']:
            raise ValueError('repository snapshots required')
        upstreams, paths = set(), set()
        for file in source['file']:
            fields(file, {'upstream', 'path', 'sha256'})
            for value in file.values():
                text(value)
            if Path(file['upstream']).is_absolute() or '..' in Path(file['upstream']).parts or file['upstream'] in upstreams or file['path'] in paths:
                raise ValueError('invalid or duplicate snapshot path')
            upstreams.add(file['upstream'])
            paths.add(file['path'])
            if not re.fullmatch(r'[0-9a-f]{64}', file['sha256']) or hashlib.sha256(local_path(file['path']).read_bytes()).hexdigest() != file['sha256']:
                raise ValueError(f'snapshot digest mismatch: {file["path"]}')
        if source['workflow'] not in upstreams or not any(Path(p).name.startswith('LICENSE') for p in upstreams):
            raise ValueError('workflow and license snapshots required')
    return data['repository']


def native_python(case):
    """Run only our owned dependency-free reference, never upstream source code."""
    with tempfile.TemporaryDirectory(prefix='wsr-reference-') as directory:
        project = Path(directory) / 'project'
        shutil.copytree(local_path(case['input']), project, ignore=shutil.ignore_patterns('__pycache__'))
        env = os.environ.copy()
        env['WSR_EXAMPLE_MODE'] = case['mode']
        result = subprocess.run([sys.executable, *NATIVE_COMMAND], cwd=project, env=env,
                                text=True, capture_output=True, timeout=30, check=False)
        output = result.stdout + result.stderr
        expected = 0 if case['mode'] == 'pass' else 1
        marker = 'OK' if expected == 0 else 'controlled WSR example assertion'
        if result.returncode != expected or 'Ran 1 test' not in output or 'test_addition' not in output or marker not in output:
            raise ValueError(f'{case["id"]}: expected native exit {expected} and test evidence; got {result.returncode}\n{output}')
        return {'id': case['id'], 'outcome': 'passed', 'layer': 'native-reference',
                'workload_exit': result.returncode, 'python': sys.version.split()[0]}


def run_cases(cases, selected=None):
    by_id = {case['id']: case for case in cases}
    for case_id in REQUIRED:
        if case_id not in by_id or by_id[case_id]['status'] != 'required':
            raise ValueError(f'required baseline case missing or downgraded: {case_id}')
        if (by_id[case_id]['kind'], by_id[case_id].get('mode'), by_id[case_id]['input']) != ('native-python', REQUIRED[case_id], 'examples/python-basic'):
            raise ValueError(f'required baseline contract changed: {case_id}')
    chosen = [by_id[selected]] if selected in by_id else []
    if selected and not chosen:
        raise ValueError(f'unknown case: {selected}')
    if not selected:
        chosen = [case for case in cases if case['status'] == 'required']
    results = []
    for case in chosen:
        if case['status'] == 'planned':
            raise ValueError(f'{case["id"]}: planned/not run; WSR behavior and executable acceptance contract are pending')
        if case['kind'] != 'native-python':
            raise ValueError(f'{case["id"]}: no executable driver; cannot claim validation')
        results.append(native_python(case))
    return results


def prepare(source):
    """Fetch pinned Git objects explicitly; do not checkout or execute upstream code."""
    parent = ROOT / 'target/wsr-tests/sources'
    parent.mkdir(parents=True, exist_ok=True)
    destination = parent / f'{source["name"]}-{source["commit"]}.git'
    if destination.exists():
        raise ValueError(f'source cache already exists: {destination}; remove it explicitly to prepare again')
    with tempfile.TemporaryDirectory(dir=parent, prefix='.staging-') as directory:
        staged = Path(directory) / 'source.git'
        def git(*args):
            return subprocess.run(['git', '-c', 'core.hooksPath=/dev/null', *args],
                                  text=True, capture_output=True, timeout=180, check=True).stdout.strip()
        git('init', '--bare', str(staged))
        git('-C', str(staged), 'fetch', '--depth=1', '--no-tags', source['url'], source['commit'])
        actual = git('-C', str(staged), 'rev-parse', 'FETCH_HEAD^{commit}')
        if actual != source['commit']:
            raise ValueError('fetched commit identity mismatch')
        for file in source['file']:
            content = subprocess.run(['git', '-C', str(staged), 'show', f'{actual}:{file["upstream"]}'],
                                     capture_output=True, timeout=30, check=True).stdout
            if hashlib.sha256(content).hexdigest() != file['sha256']:
                raise ValueError(f'upstream does not match snapshot: {file["upstream"]}')
        git('-C', str(staged), 'update-ref', 'refs/heads/pinned', actual)
        staged.rename(destination)
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument('--case', help='Run one case; planned WSR cases fail explicitly')
    selection.add_argument('--list', action='store_true', help='List inventory; does not execute tests')
    selection.add_argument('--prepare', metavar='REPOSITORY', help='Explicitly fetch pinned bare Git source; never executes it')
    args = parser.parse_args()
    try:
        output = ROOT / 'target/wsr-tests/results.json'
        if not args.list and not args.prepare:
            output.unlink(missing_ok=True)
        cases, sources = load_catalog(), load_sources()
        if args.list:
            for case in cases:
                print(f'{case["id"]}: {case["status"]} ({case["kind"]}) — {case["expectation"]}')
            return 0
        if args.prepare:
            source = next((s for s in sources if s['name'] == args.prepare), None)
            if source is None:
                raise ValueError(f'unknown repository: {args.prepare}')
            print(f'Prepared source only (not executed): {prepare(source)}')
            return 0
        results = run_cases(cases, args.case)
        revision = subprocess.run(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'],
                                  capture_output=True, text=True, timeout=10, check=True).stdout.strip()
        inputs = [ROOT / 'tests/cases.toml', ROOT / 'tests/ecosystem/cases.toml',
                  ROOT / '.github/workflows/example-python.yml',
                  *sorted((ROOT / 'examples/python-basic').rglob('*.py'))]
        report = {'version': 1, 'wsr_revision': revision,
                  'host': {'os': sys.platform, 'architecture': platform.machine()},
                  'input_digests': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs},
                  'results': results, 'snapshots': [s['name'] + '@' + s['commit'] for s in sources],
                  'not_run': [c['id'] for c in cases if c['id'] not in {r['id'] for r in results}],
                  'scope': 'catalog/snapshot integrity and native references; WSR runtime/security not exercised'}
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report, indent=2))
        return 0
    except (ValueError, OSError, subprocess.SubprocessError, tomllib.TOMLDecodeError) as error:
        print(f'corpus failure: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
