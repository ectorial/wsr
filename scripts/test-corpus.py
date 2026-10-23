#!/usr/bin/env python3
"""Regression tests for corpus validation, independent of WSR runtime support."""
import copy
import hashlib
import subprocess
import tempfile
from unittest.mock import patch
import unittest
from pathlib import Path

import corpus


class CatalogTests(unittest.TestCase):
    def load(self, text):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'cases.toml'
            path.write_text(text)
            return corpus.load_catalog(path)

    def test_duplicate_ids_are_rejected(self):
        case = '\n[[case]]\nid="REF-001"\nstatus="planned"\nkind="inspection"\ninput="tests/fixtures/inspection/basic.yml"\nexpectation="one job"\n'
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            self.load('version=1\n' + case + case)

    def test_unknown_fields_and_missing_inputs_are_rejected(self):
        base = 'version=1\n[[case]]\nid="REF-001"\nstatus="planned"\nkind="inspection"\ninput="tests/fixtures/inspection/basic.yml"\nexpectation="one job"\n'
        for text, message in [(base + 'surprise=true\n', 'invalid fields'),
                              (base.replace('basic.yml', 'absent.yml'), 'missing'),
                              (base.replace('planned', 'disabled'), 'invalid case status')]:
            with self.subTest(message=message), self.assertRaisesRegex(ValueError, message):
                self.load(text)

    def test_required_baseline_cannot_disappear_or_be_downgraded(self):
        cases = corpus.load_catalog()
        for changed in [[c for c in cases if c['id'] != 'REF-PY-PASS'],
                        [dict(c, status='planned') if c['id'] == 'REF-PY-PASS' else c for c in cases]]:
            with self.assertRaisesRegex(ValueError, 'missing or downgraded'):
                corpus.run_cases(changed)

    def test_required_failure_reference_cannot_become_a_second_pass(self):
        cases = corpus.load_catalog()
        for case in cases:
            if case['id'] == 'REF-PY-FAIL':
                case['mode'] = 'pass'
        with self.assertRaisesRegex(ValueError, 'baseline contract'):
            corpus.run_cases(cases)

    def test_planned_case_selection_is_not_success(self):
        with patch.object(corpus, 'native_python') as execute:
            with self.assertRaisesRegex(ValueError, 'planned/not run'):
                corpus.run_cases(corpus.load_catalog(), 'INS-001')
            execute.assert_not_called()

    def test_zero_tests_or_wrong_failure_cannot_satisfy_reference(self):
        case = next(c for c in corpus.load_catalog() if c['id'] == 'REF-PY-FAIL')
        for output in ['Ran 0 tests\nOK', 'Ran 1 test\ntest_addition\nImportError']:
            result = subprocess.CompletedProcess([], 1, '', output)
            with patch.object(corpus.subprocess, 'run', return_value=result):
                with self.assertRaisesRegex(ValueError, 'expected native exit'):
                    corpus.native_python(case)

    def test_upstream_snapshot_modification_is_rejected(self):
        path = corpus.ROOT / 'tests/ecosystem/cases.toml'
        text = path.read_text()
        digest = corpus.load_sources()[0]['file'][0]['sha256']
        with tempfile.TemporaryDirectory() as directory:
            changed = Path(directory) / 'cases.toml'
            changed.write_text(text.replace(digest, '0' * 64))
            with self.assertRaisesRegex(ValueError, 'digest mismatch'):
                corpus.load_sources(changed)

    def test_native_pass_and_controlled_failure(self):
        results = corpus.run_cases(corpus.load_catalog())
        self.assertEqual([r['workload_exit'] for r in results], [0, 1])
        self.assertTrue(all(r['layer'] == 'native-reference' for r in results))

    def test_manual_workflow_runs_the_same_native_command(self):
        workflow = (corpus.ROOT / '.github/workflows/example-python.yml').read_text()
        self.assertIn('run: python ' + ' '.join(corpus.NATIVE_COMMAND), workflow)
        self.assertIn('working-directory: examples/python-basic', workflow)
        self.assertIn('workflow_dispatch:', workflow)
        self.assertNotIn('continue-on-error', workflow)

    def test_prepare_verifies_commit_and_snapshot_without_checkout(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            upstream = root / 'upstream'
            upstream.mkdir()
            def git(*args):
                return subprocess.run(['git', '-C', str(upstream), *args], check=True,
                                      capture_output=True, text=True).stdout.strip()
            git('init')
            (upstream / 'workflow.yml').write_text('fixture snapshot\n')
            git('add', '.')
            git('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '-m', 'fixture')
            sha = git('rev-parse', 'HEAD')
            source = {'name': 'fixture', 'url': str(upstream), 'commit': sha,
                      'file': [{'upstream': 'workflow.yml', 'sha256': hashlib.sha256(b'fixture snapshot\n').hexdigest()}]}
            with patch.object(corpus, 'ROOT', root):
                result = corpus.prepare(source)
                self.assertTrue((result / 'objects').is_dir())
                self.assertFalse((result / 'workflow.yml').exists())
                with self.assertRaisesRegex(ValueError, 'already exists'):
                    corpus.prepare(source)
                invalid = copy.deepcopy(source)
                invalid['name'] = 'bad-digest'
                invalid['file'][0]['sha256'] = '0' * 64
                with self.assertRaisesRegex(ValueError, 'does not match'):
                    corpus.prepare(invalid)
                self.assertFalse((root / f'target/wsr-tests/sources/bad-digest-{sha}.git').exists())


if __name__ == '__main__':
    unittest.main()
