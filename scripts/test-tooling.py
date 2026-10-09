#!/usr/bin/env python3
"""Check CI isolation, dist compatibility, and release hooks without installs/uploads."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parent.parent
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
XTASK = TARGET.resolve() / "debug" / "xtask"
REAL_CARGO = shutil.which("cargo")
REAL_GIT = shutil.which("git")


def main():
    assert XTASK.is_file(), "Build xtask before running this script"
    with tempfile.TemporaryDirectory(prefix="wsr-tooling-") as temporary:
        directory = Path(temporary)
        project = directory / "project"
        crate = project / "crates/fixture"
        (crate / "src").mkdir(parents=True)
        (crate / "src/main.rs").write_text("fn main() {}\n")
        (crate / "Cargo.toml").write_text(
            '[package]\nname = "fixture"\nversion = "0.0.3"\nedition = "2024"\n'
        )
        (project / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["crates/fixture"]\nresolver = "3"\n'
            '[workspace.package]\nversion = "0.0.3"\n'
            '[workspace.metadata.xtask]\ncrate-prefix = "fixture"\nprimary-crate = "fixture"\n'
        )
        shutil.copy2(ROOT / "dist-workspace.toml", project / "dist-workspace.toml")
        version = tomllib.loads((project / "dist-workspace.toml").read_text())["dist"]["cargo-dist-version"]
        binaries = directory / "bin"
        binaries.mkdir()
        log = directory / "commands.jsonl"
        environment = dict(
            os.environ,
            PATH=str(binaries) + os.pathsep + os.environ["PATH"],
            CARGO=str(binaries / "cargo"),
            TOOLING_LOG=str(log),
            TOOLING_CARGO=REAL_CARGO,
            TOOLING_GIT=REAL_GIT,
            TOOLING_DIST_VERSION=version,
            RUSTDOCFLAGS="--cfg tooling_fixture",
        )
        stub = f"#!{sys.executable}\n" + '''import json, os, sys
from pathlib import Path
program = Path(sys.argv[0]).name
args = sys.argv[1:]
if program == "cargo" and args[0] == "metadata":
    os.execv(os.environ["TOOLING_CARGO"], ["cargo", *args])
if program == "dist" and args == ["--version"]:
    print("dist " + os.environ["TOOLING_DIST_VERSION"])
    sys.exit(0)
with open(os.environ["TOOLING_LOG"], "a") as log:
    log.write(json.dumps({"program": program, "args": args,
                         "rustdocflags": os.environ.get("RUSTDOCFLAGS")}) + "\\n")
if program == "git":
    if args[0] == "cliff":
        Path("CHANGELOG.md").write_text("fixture release changelog\\n")
    else:
        os.execv(os.environ["TOOLING_GIT"], ["git", *args])
'''
        for name in ["cargo", "rustup", "dist", "git"]:
            path = binaries / name
            path.write_text(stub)
            path.chmod(0o755)

        def commands():
            return [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []

        def run(*args, success=True):
            result = subprocess.run([XTASK, *args], cwd=project, env=environment,
                                    capture_output=True, text=True)
            assert (result.returncode == 0) == success, result.stdout + result.stderr
            return result

        # Each hosted job has an independent scope; ordinary local CI still covers every stage.
        for job, expected in [
            ("format", ["fmt"]), ("lint", ["check", "clippy"]),
            ("docs", ["doc"]), ("test", ["test"]),
            (None, ["fmt", "check", "clippy", "test", "doc"]),
        ]:
            log.unlink(missing_ok=True)
            run("ci", *(["--job", job] if job else []))
            recorded = commands()
            assert [command["args"][0] for command in recorded] == expected
            for command in recorded:
                args = command["args"]
                if args[0] != "fmt":
                    assert "--locked" in args
                if args[0] in ["check", "clippy"]:
                    assert "--all-targets" in args and "--all-features" in args
                if args[0] == "doc":
                    assert "--cfg tooling_fixture" in command["rustdocflags"]
                    assert "-D warnings" in command["rustdocflags"]
        before = commands()
        run("ci", "--job", "test", "--full", success=False)
        assert commands() == before, "conflicting CI options executed commands"

        # An incompatible local dist must not mask a compatible PATH executable.
        local = project / ".xtask/tools/bin/dist"
        local.parent.mkdir(parents=True)
        local.write_text(f"#!{sys.executable}\nprint('dist 0.0.0')\n")
        local.chmod(0o755)
        log.unlink()
        run("release", "plan", "--tag=fixture-v0.0.3")
        assert commands()[0]["program"] == "dist"
        environment["TOOLING_DIST_VERSION"] = "0.0.0"
        log.unlink()
        run("release", "plan", success=False)
        assert commands() == [], "an incompatible dist executed release planning"
        environment["TOOLING_DIST_VERSION"] = version
        run("tools", "sync", "release")
        installs = [command["args"] for command in commands() if command["program"] == "cargo"]
        dist = next(args for args in installs if args[-1] == "cargo-dist")
        assert "--locked" in dist and dist[dist.index("--tag") + 1] == "v" + version
        assert "--git" in dist

        # Exercise the actual configured hook, including index preservation in previews.
        changelog = project / "CHANGELOG.md"
        changelog.write_text("existing release history\n")
        subprocess.run([REAL_GIT, "init", "--quiet"], cwd=project, check=True)
        subprocess.run([REAL_GIT, "add", "CHANGELOG.md"], cwd=project, check=True)
        index = project / ".git/index"
        before = changelog.read_bytes(), index.read_bytes()
        hook = tomllib.loads((ROOT / "release.toml").read_text())["pre-release-hook"]
        hook = [argument.replace("{{version}}", "0.0.4") for argument in hook]
        for dry_run in ["true", "false"]:
            log.unlink(missing_ok=True)
            subprocess.run(hook, cwd=project, env=dict(environment, DRY_RUN=dry_run), check=True)
            if dry_run == "true":
                assert (changelog.read_bytes(), index.read_bytes()) == before
                assert commands() == []
            else:
                assert changelog.read_bytes() != before[0]
                assert [command["args"][0] for command in commands()] == ["cliff", "add"]
                staged = subprocess.check_output([REAL_GIT, "show", ":CHANGELOG.md"], cwd=project)
                assert staged == changelog.read_bytes()
    print("CI isolation, configured dist, and dry-run/execute hook checks passed.")


if __name__ == "__main__":
    main()
