mod common;
use wsr_gha::Category::{Invalid, Unsupported};

const JOB: &str = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";

// workflow-description is a plain `string` with no expression context (B-33).
#[test]
fn description_text_is_retained_as_written() {
    for (value, expected) in [
        ("Builds it", Some("Builds it")),
        ("''", Some("")),
        ("'  '", Some("  ")),
        ("\"a\\u202Eb\"", Some("a\u{202E}b")),
        ("~", None),
    ] {
        let summary = common::inspect(format!("description: {value}\n{JOB}").as_bytes())
            .unwrap_or_else(|error| panic!("{value}: {error}"));
        assert_eq!(summary.description.as_deref(), expected, "{value}");
    }
}

#[test]
fn description_categories_follow_the_pinned_schema() {
    for (value, category) in [
        ("5", Unsupported),
        ("true", Unsupported),
        ("'${{ github.ref }}'", Invalid),
        ("[a]", Invalid),
        ("{a: b}", Invalid),
    ] {
        let Err(error) = common::inspect(format!("description: {value}\n{JOB}").as_bytes()) else {
            panic!("accepted description: {value}");
        };
        assert_eq!(
            (error.category(), error.route()),
            (category, Some("description")),
            "{value}: {error}"
        );
    }
}
