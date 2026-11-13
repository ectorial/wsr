//! Backlog B-18: job `outputs` kept as source text, never resolved.

use std::collections::BTreeMap;

use wsr_gha::Category;
use wsr_types::inspection::SourceText;

fn job(outputs: &str) -> String {
    format!(
        "jobs:\n  b:\n    runs-on: linux\n    outputs: {outputs}\n    steps:\n      - run: echo\n"
    )
}

#[test]
fn outputs_are_kept_by_name() {
    let summary =
        wsr_gha::inspect(job("{z: '${{ steps.s.outputs.r }}', a: literal, M: ''}").as_bytes())
            .unwrap();
    assert_eq!(
        summary.jobs[0].outputs,
        Some(BTreeMap::from([
            ("M".into(), SourceText::Literal(String::new())),
            ("a".into(), SourceText::Literal("literal".into())),
            (
                "z".into(),
                SourceText::Unevaluated("${{ steps.s.outputs.r }}".into())
            ),
        ]))
    );
    assert_eq!(
        wsr_gha::inspect(job("{}").as_bytes()).unwrap().jobs[0].outputs,
        Some(BTreeMap::new())
    );
    let without = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";
    assert_eq!(
        wsr_gha::inspect(without.as_bytes()).unwrap().jobs[0].outputs,
        None
    );
}

fn assert_category(outputs: &str, category: Category, route: &str) {
    let error = wsr_gha::inspect(job(outputs).as_bytes()).unwrap_err();
    assert_eq!(
        (error.category(), error.route()),
        (category, Some(route)),
        "{outputs}: {error}"
    );
}

#[test]
fn schema_rejected_outputs_are_invalid() {
    for (outputs, route) in [
        // job-outputs has no expression context, for the mapping or its keys.
        ("'${{ fromJSON(x) }}'", "jobs.b.outputs"),
        ("{'${{ x }}': y}", "jobs.b.outputs"),
        ("[a]", "jobs.b.outputs"),
        ("~", "jobs.b.outputs"),
        ("{'': x}", "jobs.b.outputs"),
        ("{a: x, A: y}", "jobs.b.outputs.A"),
        ("{a: [x]}", "jobs.b.outputs.a"),
        ("{a: {b: c}}", "jobs.b.outputs.a"),
    ] {
        assert_category(outputs, Category::Invalid, route);
    }
}

#[test]
fn converted_scalars_are_unsupported() {
    for (outputs, route) in [
        ("{a: 5}", "jobs.b.outputs.a"),
        ("{a: true}", "jobs.b.outputs.a"),
        ("{a: ~}", "jobs.b.outputs.a"),
        ("{5: x}", "jobs.b.outputs"),
    ] {
        assert_category(outputs, Category::Unsupported, route);
    }
}
