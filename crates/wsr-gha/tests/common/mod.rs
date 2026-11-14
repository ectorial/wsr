//! Shared fixture helper (backlog B-25). The pinned schema requires `on`; a fixture whose root
//! mapping lacks it gets `on: workflow_dispatch` appended after its content, so parser locations
//! in its diagnostics are unchanged. Other sources are inspected as written.

pub fn inspect(source: &[u8]) -> Result<wsr_types::inspection::WorkflowSummary, wsr_gha::Error> {
    let missing = matches!(
        serde_yaml::from_slice::<serde_yaml::Value>(source),
        Ok(serde_yaml::Value::Mapping(root)) if !root.contains_key("on")
    );
    if !missing {
        return wsr_gha::inspect(source);
    }
    let mut source = source.to_vec();
    if !source.ends_with(b"\n") {
        source.push(b'\n');
    }
    source.extend_from_slice(b"on: workflow_dispatch\n");
    wsr_gha::inspect(&source)
}
