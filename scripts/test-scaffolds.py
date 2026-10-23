#!/usr/bin/env python3
"""Verify template test obligations in disposable workspaces."""

from pathlib import Path
import os
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
XTASK = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target')).resolve() / 'debug/xtask'


def run(directory, *arguments):
    return subprocess.run(arguments, cwd=directory, text=True, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT)


def main():
    for kind in ['library', 'binary', 'primary', 'companion']:
        with tempfile.TemporaryDirectory(prefix='wsr-scaffold-') as temporary:
            project = Path(temporary)
            primary = project / 'crates/fixture'
            (primary / 'src').mkdir(parents=True)
            (primary / 'Cargo.toml').write_text(
                '[package]\nname = "fixture"\nversion.workspace = true\nedition.workspace = true\n'
            )
            (primary / 'src/lib.rs').write_text(
                '/// Starter message.\npub fn hello() -> &\'static str { "Hello from fixture!" }\n'
            )
            (project / 'Cargo.toml').write_text(
                '[workspace]\nmembers = ["crates/*"]\nresolver = "3"\n'
                '[workspace.package]\nversion = "0.0.3"\nedition = "2024"\n'
                'rust-version = "1.85"\nauthors = []\nlicense = "MIT"\n'
                'repository = "https://example.com"\nhomepage = "https://example.com"\n'
                '[workspace.dependencies]\nfixture = { path = "crates/fixture", version = "0.0.3" }\n'
                'clap = { version = "4.6.7", features = ["derive"] }\n'
                '[workspace.lints.rust]\nunsafe_code = "warn"\n'
                '[workspace.metadata.xtask]\ncrate-prefix = "fixture"\nprimary-crate = "fixture"\n'
            )
            if kind in ['library', 'binary']:
                arguments = ['add', 'sample', '--private']
                if kind == 'binary':
                    arguments.append('--bin')
                package = 'fixture-sample'
                targets = ['--lib', '--test=integration', '--doc'] if kind == 'library' else [
                    '--bin=fixture-sample', '--test=e2e']
            else:
                arguments = ['init', 'cli', '--entrypoint', kind]
                package = 'fixture-cli'
                targets = ['--lib', '--test=integration', '--doc']
            result = run(project, str(XTASK), *arguments)
            assert result.returncode == 0, result.stdout
            for target in targets:
                result = run(project, 'cargo', 'test', '-p', package, target)
                assert result.returncode != 0, result.stdout
                assert 'TEST_OBLIGATION:' in result.stdout, result.stdout
                assert 'test result: FAILED' in result.stdout, result.stdout
                assert 'could not compile' not in result.stdout, result.stdout
            if kind in ['primary', 'companion']:
                owner = 'fixture' if kind == 'primary' else package
                result = run(project, 'cargo', 'test', '-p', owner, '--test=e2e')
                assert result.returncode != 0 and 'TEST_OBLIGATION:' in result.stdout, result.stdout
                assert 'could not compile' not in result.stdout, result.stdout
            # Only disposable examples are completed; repository obligations stay intact.
            for source in project.glob('crates/**/*.rs'):
                source.write_text(''.join(line for line in source.read_text().splitlines(True)
                                          if 'TEST_OBLIGATION:' not in line))
            result = run(project, 'cargo', 'test', '--workspace')
            assert result.returncode == 0, result.stdout
            print(f'{kind}: intended test failures verified; completed fixture passes')


if __name__ == '__main__':
    main()
