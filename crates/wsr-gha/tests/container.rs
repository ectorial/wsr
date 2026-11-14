//! Backlog B-29: job `container` reported as written; never pulled or started.

mod common;

use std::collections::BTreeMap;

use wsr_gha::Category::{self, Invalid, Unsupported};
use wsr_types::inspection::{Container, ContainerCredentials, SourceText, StepValue};

fn literal(text: &str) -> SourceText {
    SourceText::Literal(text.into())
}

fn unevaluated(text: &str) -> SourceText {
    SourceText::Unevaluated(text.into())
}

fn job(container: &str) -> String {
    format!(
        "jobs:\n  a:\n    runs-on: linux\n    container: {container}\n    steps:\n      - run: echo\n"
    )
}

fn container(source: &str) -> Container {
    let mut summary = common::inspect(job(source).as_bytes()).unwrap();
    summary.jobs.remove(0).container.expect("a container")
}

fn image(image: SourceText) -> Container {
    Container {
        image,
        credentials: None,
        env: None,
        ports: None,
        volumes: None,
        options: None,
    }
}

#[test]
fn string_and_mapping_forms_are_kept_as_written() {
    assert_eq!(container("node:20"), image(literal("node:20")));
    assert_eq!(
        container("${{ matrix.image }}"),
        image(unevaluated("${{ matrix.image }}"))
    );
    assert_eq!(container("{image: node:20}"), image(literal("node:20")));
    assert_eq!(
        container(
            "\n      image: ghcr.io/o/i:1\n      credentials:\n        username: bot\n        password: ${{ secrets.TOKEN }}\n      env:\n        B: 2\n        A: ${{ vars.A }}\n      ports: [80, '8080:80', '${{ inputs.port }}']\n      volumes: [/data, true]\n      options: ''"
        ),
        Container {
            image: literal("ghcr.io/o/i:1"),
            credentials: Some(ContainerCredentials {
                username: Some(literal("bot")),
                password: Some(unevaluated("${{ secrets.TOKEN }}")),
            }),
            env: Some(BTreeMap::from([
                (
                    "A".to_owned(),
                    StepValue::Text(unevaluated("${{ vars.A }}"))
                ),
                ("B".to_owned(), StepValue::Number("2".into())),
            ])),
            ports: Some(vec![
                StepValue::Number("80".into()),
                StepValue::Text(literal("8080:80")),
                StepValue::Text(unevaluated("${{ inputs.port }}")),
            ]),
            volumes: Some(vec![
                StepValue::Text(literal("/data")),
                StepValue::Boolean(true),
            ]),
            options: Some(literal("")),
        }
    );
    assert_eq!(
        container("{image: x, credentials: {}, env: {}, ports: [], volumes: []}"),
        Container {
            image: literal("x"),
            credentials: Some(ContainerCredentials {
                username: None,
                password: None,
            }),
            env: Some(BTreeMap::new()),
            ports: Some(Vec::new()),
            volumes: Some(Vec::new()),
            options: None,
        }
    );
}

#[test]
fn container_categories_follow_the_pinned_schema_and_converter() {
    let route = "jobs.a.container";
    for (source, category, at) in [
        ("''", Unsupported, route.to_owned()),
        ("'  '", Unsupported, route.to_owned()),
        ("~", Unsupported, route.to_owned()),
        ("5", Unsupported, route.to_owned()),
        ("true", Unsupported, route.to_owned()),
        ("[node]", Invalid, route.to_owned()),
        ("{}", Invalid, route.to_owned()),
        ("{options: --rm}", Invalid, route.to_owned()),
        // The converter skips its checks when the container holds an expression.
        ("{options: '${{ x }}'}", Unsupported, route.to_owned()),
        (
            "{'${{ x }}': y, image: i}",
            Unsupported,
            format!("{route}.${{{{ x }}}}"),
        ),
        ("{image: i, bogus: 1}", Invalid, format!("{route}.bogus")),
        ("{image: i, Image: j}", Invalid, format!("{route}.Image")),
        ("{image: i, 5: j}", Invalid, route.to_owned()),
        ("{image: ''}", Invalid, format!("{route}.image")),
        ("{image: ' '}", Invalid, format!("{route}.image")),
        ("{image: ~}", Invalid, format!("{route}.image")),
        ("{image: 5}", Unsupported, format!("{route}.image")),
        ("{image: [i]}", Invalid, format!("{route}.image")),
        (
            "{image: i, options: 5}",
            Unsupported,
            format!("{route}.options"),
        ),
        (
            "{image: i, options: [a]}",
            Invalid,
            format!("{route}.options"),
        ),
        ("{image: i, env: ~}", Invalid, format!("{route}.env")),
        (
            "{image: i, env: '${{ x }}'}",
            Unsupported,
            format!("{route}.env"),
        ),
        (
            "{image: i, env: {a: 1, A: 2}}",
            Invalid,
            format!("{route}.env.A"),
        ),
        (
            "{image: i, ports: '${{ x }}'}",
            Unsupported,
            format!("{route}.ports"),
        ),
        ("{image: i, ports: 80}", Invalid, format!("{route}.ports")),
        (
            "{image: i, ports: ['']}",
            Invalid,
            format!("{route}.ports[1]"),
        ),
        (
            "{image: i, ports: [80, ~]}",
            Invalid,
            format!("{route}.ports[2]"),
        ),
        (
            "{image: i, volumes: [[a]]}",
            Invalid,
            format!("{route}.volumes[1]"),
        ),
        (
            "{image: i, credentials: '${{ x }}'}",
            Unsupported,
            format!("{route}.credentials"),
        ),
        (
            "{image: i, credentials: [a]}",
            Invalid,
            format!("{route}.credentials"),
        ),
        (
            "{image: i, credentials: {token: a}}",
            Invalid,
            format!("{route}.credentials.token"),
        ),
        (
            "{image: i, credentials: {username: ''}}",
            Invalid,
            format!("{route}.credentials.username"),
        ),
        (
            "{image: i, credentials: {password: 5}}",
            Unsupported,
            format!("{route}.credentials.password"),
        ),
        (
            "{image: i, credentials: {'${{ x }}': a}}",
            Unsupported,
            format!("{route}.credentials.${{{{ x }}}}"),
        ),
    ] {
        let error = common::inspect(job(source).as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (category, Some(at.as_str())),
            "{source}: {error}"
        );
    }
}

#[test]
fn container_with_uses_stays_invalid() {
    let error =
        common::inspect(b"jobs:\n  a:\n    uses: ./w.yml\n    container: node\n").unwrap_err();
    assert_eq!(error.category(), Category::Invalid, "{error}");
}

// Past the B-23 scan budget, a `container` read before `uses` is now decoded, so the call check
// rejects it instead of leaving it unreported.
#[test]
fn container_before_uses_is_invalid_when_the_scan_budget_is_exhausted() {
    let mut source = "jobs:\n  x: &x\n    name: n\n    if: always()\n    runs-on: linux\n    timeout-minutes: 5\n    continue-on-error: false\n    env: {A: b}\n    defaults: {run: {shell: bash}}\n    outputs: {o: v}\n    environment: prod\n    permissions: read-all\n    concurrency: g\n    needs: []\n    steps: [{run: echo}]\n".to_owned();
    for index in 0..2500 {
        source.push_str(&format!("  a{index}: *x\n"));
    }
    source.push_str("  z: {container: node, uses: ./x.yml}\n");
    let Err(error) = common::inspect(source.as_bytes()) else {
        panic!("inspection succeeded");
    };
    assert_eq!(
        (error.category(), error.route()),
        (Category::Invalid, Some("jobs.z.container")),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains("`container` cannot be combined with `uses`"),
        "{error}"
    );
}
