//! Owning-crate characterization of the static inspection compiler, independent of host rendering.

use wsr_gha::Category;
use wsr_types::inspection::{RunnerRequirement, StepSummaryKind};

fn job(id: &str, needs: &str) -> String {
    format!("  {id}:\n    runs-on: linux\n{needs}    steps:\n      - run: echo\n")
}

fn workflow(jobs: &[(&str, &str)]) -> String {
    let mut source = "jobs:\n".to_owned();
    for (id, needs) in jobs {
        source.push_str(&job(id, needs));
    }
    source
}

fn failure(source: &str) -> (Category, Option<String>, String) {
    let error = wsr_gha::inspect(source.as_bytes()).unwrap_err();
    (
        error.category(),
        error.route().map(str::to_owned),
        error.to_string(),
    )
}

#[test]
fn dependency_graph_accepts_diamonds_repeated_edges_and_forward_references() {
    let summary = wsr_gha::inspect(
        workflow(&[
            ("a", "    needs: [b, c, b]\n"),
            ("b", "    needs: d\n"),
            ("c", "    needs: [d]\n"),
            ("d", "    needs: []\n"),
        ])
        .as_bytes(),
    )
    .unwrap();
    let edges = summary
        .jobs
        .iter()
        .map(|job| (job.id.as_str(), job.needs.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        edges,
        [
            ("a", vec!["b".to_owned(), "c".to_owned(), "b".to_owned()]),
            ("b", vec!["d".to_owned()]),
            ("c", vec!["d".to_owned()]),
            ("d", vec![]),
        ]
    );
}

#[test]
fn dependency_graph_rejects_missing_self_and_multi_job_cycles_at_their_routes() {
    for (jobs, route, detail) in [
        (
            vec![("a", "    needs: [b, missing]\n"), ("b", "")],
            "jobs.a.needs[2]",
            "unknown job 'missing'",
        ),
        (
            vec![("a", "    needs: a\n")],
            "jobs.a.needs",
            "job 'a' depends on itself",
        ),
        (
            vec![
                ("a", "    needs: [b]\n"),
                ("b", "    needs: [c]\n"),
                ("c", "    needs: [a]\n"),
            ],
            "jobs.c.needs[1]",
            "dependency cycle: a -> b -> c -> a",
        ),
    ] {
        let (category, actual, message) = failure(&workflow(&jobs));
        assert_eq!(category, Category::Invalid, "{message}");
        assert_eq!(actual.as_deref(), Some(route), "{message}");
        assert_eq!(message, format!("{route}: {detail}"));
    }
}

#[test]
fn job_and_step_identifier_rules_are_enforced_before_dependencies() {
    for (source, category, route) in [
        (
            workflow(&[("'b c'", "    needs: missing\n")]),
            Category::Invalid,
            "jobs.b c",
        ),
        (
            workflow(&[("'1b'", "")]),
            Category::Invalid,
            "jobs.1b",
        ),
        (
            workflow(&[("'${{ x }}'", "")]),
            Category::Unsupported,
            "jobs.${{ x }}",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - id: __x\n        run: a\n"
                .to_owned(),
            Category::Invalid,
            "jobs.b.steps[1].id",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - id: s\n        run: a\n      - id: s\n        run: b\n"
                .to_owned(),
            Category::Invalid,
            "jobs.b.steps[2].id",
        ),
    ] {
        let (actual_category, actual, message) = failure(&source);
        assert_eq!(
            (actual_category, actual.as_deref()),
            (category, Some(route)),
            "{message}"
        );
    }
}

#[test]
fn summaries_preserve_runner_forms_and_step_kinds() {
    let summary = wsr_gha::inspect(
        b"jobs:\n  b:\n    runs-on: [self-hosted, linux, linux]\n    steps:\n      - id: one\n        run: echo\n      - uses: owner/action@v1\n        name: Use\n",
    )
    .unwrap();
    let job = &summary.jobs[0];
    assert!(
        matches!(&job.runner, RunnerRequirement::Labels(labels) if labels == &["self-hosted", "linux", "linux"]),
        "{:?}",
        job.runner
    );
    let [first, second] = &job.steps[..] else {
        panic!("expected two steps");
    };
    assert_eq!(
        (first.id.as_deref(), first.name.as_deref()),
        (Some("one"), None)
    );
    assert!(matches!(first.kind, StepSummaryKind::Run));
    assert_eq!(
        (second.id.as_deref(), second.name.as_deref()),
        (None, Some("Use"))
    );
    assert!(
        matches!(&second.kind, StepSummaryKind::Uses(reference) if reference == "owner/action@v1")
    );
}

#[test]
fn shape_errors_report_invalid_routes() {
    for (source, route) in [
        ("jobs: {}", Some("jobs")),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps: []\n",
            Some("jobs.b.steps"),
        ),
        (
            "jobs:\n  b:\n    runs-on: []\n    steps:\n      - run: a\n",
            Some("jobs.b.runs-on"),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: a\n        uses: b\n",
            Some("jobs.b.steps[1]"),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    needs: {a: 1}\n    steps:\n      - run: a\n",
            Some("jobs.b.needs"),
        ),
        // Raised by the YAML decoder itself, so no typed route is attached.
        ("jobs:\n  b:\n    runs-on: linux\n    steps: [5]\n", None),
    ] {
        let (category, actual, message) = failure(source);
        assert_eq!(
            (category, actual.as_deref()),
            (Category::Invalid, route),
            "{source}: {message}"
        );
    }
}

#[test]
fn hostile_yaml_fails_without_panicking() {
    // Each anchor repeats the previous one ten times; `needs` decodes its value, forcing expansion.
    let mut bomb = "[&a [x, x, x, x, x, x, x, x, x, x]".to_owned();
    for level in 1..10u8 {
        let (previous, current) = ((b'a' + level - 1) as char, (b'a' + level) as char);
        bomb.push_str(&format!(
            ", &{current} [{}]",
            vec![format!("*{previous}"); 10].join(", ")
        ));
    }
    bomb.push(']');
    let nested = format!("{}{}", "[".repeat(10_000), "]".repeat(10_000));
    for (source, detail) in [
        (
            format!(
                "jobs:\n  b:\n    runs-on: linux\n    needs: {bomb}\n    steps:\n      - run: a\n"
            ),
            "repetition limit exceeded",
        ),
        // A non-mapping root is rejected by shape before its nesting is walked.
        (
            nested.clone(),
            "invalid type: sequence, expected struct Workflow",
        ),
        (
            format!(
                "jobs:\n  b:\n    runs-on: linux\n    needs: {nested}\n    steps:\n      - run: a\n"
            ),
            "recursion limit exceeded",
        ),
    ] {
        let (category, route, message) = failure(&source);
        assert_eq!((category, route), (Category::Invalid, None), "{message}");
        assert!(message.contains(detail), "{message}");
    }
}
