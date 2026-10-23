//! Corpus integrity and native references, independent of unimplemented WSR execution.

use std::path::Path;
use std::process::Command;

#[test]
fn validation_corpus_foundation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for script in ["scripts/test-corpus.py", "scripts/corpus.py"] {
        let output = Command::new("python3")
            .arg(root.join(script))
            .current_dir(std::env::temp_dir())
            .output()
            .expect("corpus tests require Python 3.11+ on PATH");
        assert!(
            output.status.success(),
            "{script}:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
