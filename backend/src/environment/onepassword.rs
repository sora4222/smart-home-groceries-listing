//! The sole backend adapter for the 1Password CLI; authentication stays in `op`.
use std::process::{Command, Stdio};

/// Read a reference through desktop integration or an exported service token.
pub(super) fn read(reference: &str) -> anyhow::Result<String> {
    let output = Command::new("op")
        .args(["read", "--no-newline", reference])
        .stdin(Stdio::null())
        .output()?;
    anyhow::ensure!(output.status.success(), "Secret provider failed");
    Ok(String::from_utf8(output.stdout)?)
}
