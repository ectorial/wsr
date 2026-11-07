use wsr_types::inspection::{DispatchBody, DispatchInputType, InputDefault, ManualTrigger};

fn summary(trigger: &str) -> wsr_types::inspection::WorkflowSummary {
    wsr_gha::inspect(
        format!(
            "{trigger}jobs:\n  build:\n    runs-on: linux\n    steps:\n      - run: echo hello\n"
        )
        .as_bytes(),
    )
    .unwrap()
}

#[test]
fn dispatch_source_states_remain_distinct() {
    assert_eq!(summary("").trigger, None);
    assert_eq!(
        summary("on: workflow_dispatch\n").trigger,
        Some(ManualTrigger::Scalar)
    );
    assert_eq!(
        summary("on: {workflow_dispatch: null}\n").trigger,
        Some(ManualTrigger::Mapping(None))
    );
    assert_eq!(
        summary("on: {workflow_dispatch: {}}\n").trigger,
        Some(ManualTrigger::Mapping(Some(DispatchBody { inputs: None })))
    );
    assert_eq!(
        summary("on: {workflow_dispatch: {inputs: {}}}\n").trigger,
        Some(ManualTrigger::Mapping(Some(DispatchBody {
            inputs: Some(vec![])
        })))
    );
}

#[test]
fn dispatch_summary_retains_declared_types_defaults_and_omissions() {
    let summary = summary(
        "on: {workflow_dispatch: {inputs: {z: {type: number, default: 42}, a: {type: boolean, default: false, required: false}, m: {default: '42'}, empty: {}}}}\n",
    );
    let Some(ManualTrigger::Mapping(Some(body))) = summary.trigger else {
        panic!("expected mapping body");
    };
    let inputs = body.inputs.unwrap();
    assert_eq!(
        inputs.iter().map(|v| v.id.as_str()).collect::<Vec<_>>(),
        vec!["a", "empty", "m", "z"]
    );
    assert_eq!(inputs[0].input_type, Some(DispatchInputType::Boolean));
    assert_eq!(inputs[0].default, Some(InputDefault::Boolean(false)));
    assert_eq!(inputs[0].required, Some(false));
    assert_eq!(inputs[1].default, None);
    assert_eq!(inputs[1].required, None);
    assert_eq!(inputs[2].input_type, None);
    assert_eq!(inputs[2].default, Some(InputDefault::String("42".into())));
    assert_eq!(inputs[3].input_type, Some(DispatchInputType::Number));
    assert_eq!(inputs[3].default, Some(InputDefault::Number("42".into())));
}
