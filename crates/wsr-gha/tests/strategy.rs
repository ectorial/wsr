mod common;

#[test]
fn strategy_is_inspected_without_expanding_jobs() {
    let summary = common::inspect(b"jobs:\n  a:\n    runs-on: linux\n    strategy: {matrix: {os: [linux, macos]}}\n    steps: [{run: touch executed}]\n").expect("strategy inspection");
    assert_eq!(summary.jobs.len(), 1);
}

fn inspect_strategy(text: &str) -> Result<wsr_types::inspection::Strategy, wsr_gha::Error> {
    let source = format!("jobs:\n  a:\n    uses: ./missing.yml\n    strategy: {text}\n");
    Ok(common::inspect(source.as_bytes())?
        .jobs
        .remove(0)
        .strategy
        .unwrap())
}

#[test]
fn strategy_categories_follow_schema() {
    use wsr_gha::Category::{Invalid, Unsupported};
    for source in [
        "{}",
        "${{ needs.s }}",
        "{fail-fast: false, max-parallel: -2}",
        "{fail-fast: '${{ x }}', max-parallel: '${{ x }}'}",
        "{matrix: '${{ fromJson(x) }}'}",
        "{matrix: {a: [null, true, 2, [x], {nested: [x]}], include: [{a: {x: 1}}], exclude: []}}",
        "{matrix: {a: '${{ x }}', include: '${{ x }}', exclude: ['${{ x }}']}}",
        "{matrix: {' ': []}}",
    ] {
        assert!(inspect_strategy(source).is_ok(), "{source}");
    }
    for (source, category, route) in [
        ("null", Invalid, ""),
        ("[]", Invalid, ""),
        ("true", Invalid, ""),
        ("{bogus: 1}", Invalid, ".bogus"),
        ("{'${{ x }}': 1}", Unsupported, ".${{ x }}"),
        ("{5: x}", Invalid, ""),
        ("{fail-fast: 'true'}", Invalid, ".fail-fast"),
        ("{fail-fast: 1}", Invalid, ".fail-fast"),
        ("{fail-fast: null}", Invalid, ".fail-fast"),
        ("{max-parallel: '2'}", Invalid, ".max-parallel"),
        ("{max-parallel: false}", Invalid, ".max-parallel"),
        ("{max-parallel: null}", Invalid, ".max-parallel"),
        ("{matrix: []}", Invalid, ".matrix"),
        ("{matrix: {a: 1}}", Invalid, ".matrix.a"),
        ("{matrix: {'': []}}", Invalid, ".matrix."),
        ("{matrix: {true: []}}", Invalid, ".matrix"),
        ("{matrix: {include: {a: 1}}}", Invalid, ".matrix.include"),
        ("{matrix: {exclude: [1]}}", Invalid, ".matrix.exclude[1]"),
        (
            "{matrix: {include: [{'': 1}]}}",
            Invalid,
            ".matrix.include[1]",
        ),
        (
            "{matrix: {exclude: [{true: 1}]}}",
            Invalid,
            ".matrix.exclude[1]",
        ),
    ] {
        let error = inspect_strategy(source).unwrap_err();
        assert_eq!(error.category(), category, "{source}: {error}");
        assert_eq!(
            error.route(),
            Some(format!("jobs.a.strategy{route}").as_str()),
            "{source}: {error}"
        );
    }
}

#[test]
fn matrix_values_and_expressions_are_retained() {
    use wsr_types::inspection::{SourceText, Strategy};
    assert_eq!(
        inspect_strategy("{matrix: '${{ x }}'}").unwrap(),
        Strategy::Mapping {
            fail_fast: None,
            max_parallel: None,
            matrix: Some(SourceText::Unevaluated("${{ x }}".into()))
        }
    );
    let Strategy::Mapping {
        matrix: Some(SourceText::Literal(text)),
        ..
    } = inspect_strategy(
        "{matrix: {a: [null, true, 2, [x], {nested: y}], include: [{z: q}], exclude: []}}",
    )
    .unwrap()
    else {
        panic!("matrix source");
    };
    assert_eq!(
        text,
        "a:\n- null\n- true\n- 2\n- - x\n- nested: y\ninclude:\n- z: q\nexclude: []\n"
    );
}

#[test]
fn call_strategy_is_retained_past_scan_budget() {
    let mut source = "jobs:\n  x: &x\n    name: n\n    if: always()\n    runs-on: linux\n    timeout-minutes: 5\n    continue-on-error: false\n    env: {A: b}\n    defaults: {run: {shell: bash}}\n    outputs: {o: v}\n    environment: prod\n    permissions: read-all\n    concurrency: g\n    needs: []\n    steps: [{run: echo}]\n".to_owned();
    for index in 0..2500 {
        source.push_str(&format!("  a{index}: *x\n"));
    }
    source.push_str("  z: {strategy: {matrix: {a: [1, 2]}}, uses: ./missing.yml}\n");
    let summary = common::inspect(source.as_bytes()).unwrap();
    let z = summary.jobs.iter().find(|job| job.id == "z").unwrap();
    assert!(z.call.is_some());
    assert!(z.strategy.is_some());
    assert_eq!(summary.jobs.len(), 2502);
}

#[test]
fn decoder_errors_do_not_quote_matrix_values() {
    let error = inspect_strategy("{matrix: {a: [!!int private-value]}}").unwrap_err();
    assert_eq!(error.category(), wsr_gha::Category::Invalid);
    assert!(!error.to_string().contains("private-value"));
    assert!(error.to_string().contains("detail withheld"));
}
