//! The sole backend CLI adapter, with service-account then desktop authentication.
use std::process::{Command, Stdio};
use std::{collections::HashMap, ffi::OsString};

/// Try an exported service token, then retry once with desktop app integration.
pub(super) fn read(reference: &str) -> anyhow::Result<String> {
    read_with_environment(reference, std::env::vars_os().collect())
}

fn read_with_environment(
    reference: &str,
    inherited: HashMap<OsString, OsString>,
) -> anyhow::Result<String> {
    let mut desktop = inherited.clone();
    desktop.retain(|key, _| {
        let key = key.to_string_lossy();
        key != "OP_SERVICE_ACCOUNT_TOKEN"
            && key != "OP_SESSION"
            && !key.starts_with("OP_CONNECT_")
            && !key.starts_with("OP_SESSION_")
    });
    desktop.insert("OP_BIOMETRIC_UNLOCK_ENABLED".into(), "true".into());
    if let Some(token) = inherited.get(&OsString::from("OP_SERVICE_ACCOUNT_TOKEN")) {
        if !token.is_empty() {
            let mut service = desktop.clone();
            service.insert("OP_SERVICE_ACCOUNT_TOKEN".into(), token.clone());
            service.insert("OP_BIOMETRIC_UNLOCK_ENABLED".into(), "false".into());
            service.remove(&OsString::from("OP_ACCOUNT"));
            if let Ok(value) = read_using(reference, service) {
                return Ok(value);
            }
        }
    }
    read_using(reference, desktop)
}

fn read_using(reference: &str, environment: HashMap<OsString, OsString>) -> anyhow::Result<String> {
    let output = Command::new("op")
        .args(["read", "--no-newline", reference])
        .env_clear()
        .envs(environment)
        .stdin(Stdio::null())
        .output()?;
    anyhow::ensure!(output.status.success(), "Secret provider failed");
    let value = String::from_utf8(output.stdout)?;
    anyhow::ensure!(
        !value.is_empty() && !value.starts_with("op://"),
        "Unresolved 1Password value"
    );
    Ok(value)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn cli_authentication_tries_service_then_desktop_without_competing_credentials() {
        let dir = std::env::temp_dir().join(format!("grocery-op-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let op = dir.join("op");
        std::fs::write(
            &op,
            r##"#!/bin/sh
[ "$1" = read ] && [ "$2" = --no-newline ] || exit 2
[ -z "$OP_CONNECT_HOST$OP_CONNECT_TOKEN$OP_SESSION$OP_SESSION_fixture" ] || exit 3
if [ -n "$OP_SERVICE_ACCOUNT_TOKEN" ]; then
  [ "$OP_BIOMETRIC_UNLOCK_ENABLED" = false ] || exit 4
  printf 'service\n' >> "$AUTH_TRACE"
  case "$SERVICE_RESULT" in
    success) printf service-secret; exit 0 ;;
    empty) exit 0 ;;
    unresolved) printf op://unresolved; exit 0 ;;
  esac
  echo sensitive-provider-output >&2
  exit 7
fi
[ "$OP_BIOMETRIC_UNLOCK_ENABLED" = true ] || exit 5
printf 'desktop\n' >> "$AUTH_TRACE"
[ "$DESKTOP_DENIED" = true ] && exit 7
printf desktop-secret
"##,
        )
        .unwrap();
        std::fs::set_permissions(&op, std::fs::Permissions::from_mode(0o700)).unwrap();
        let inherited = HashMap::from([
            (OsString::from("PATH"), dir.as_os_str().to_owned()),
            ("AUTH_TRACE".into(), dir.join("trace").into_os_string()),
            ("OP_SERVICE_ACCOUNT_TOKEN".into(), "fixture-token".into()),
            ("OP_CONNECT_HOST".into(), "competing".into()),
            ("OP_CONNECT_TOKEN".into(), "competing".into()),
            ("OP_SESSION".into(), "competing".into()),
            ("OP_SESSION_fixture".into(), "competing".into()),
            ("OP_BIOMETRIC_UNLOCK_ENABLED".into(), "false".into()),
        ]);
        for (service_result, token, denied, expected, expected_trace) in [
            (
                "success",
                "token",
                "false",
                Some("service-secret"),
                "service\n",
            ),
            (
                "denied",
                "token",
                "false",
                Some("desktop-secret"),
                "service\ndesktop\n",
            ),
            (
                "empty",
                "token",
                "false",
                Some("desktop-secret"),
                "service\ndesktop\n",
            ),
            (
                "unresolved",
                "token",
                "false",
                Some("desktop-secret"),
                "service\ndesktop\n",
            ),
            ("denied", "", "false", Some("desktop-secret"), "desktop\n"),
            ("denied", "token", "true", None, "service\ndesktop\n"),
        ] {
            let mut env = inherited.clone();
            env.insert("SERVICE_RESULT".into(), service_result.into());
            env.insert("OP_SERVICE_ACCOUNT_TOKEN".into(), token.into());
            env.insert("DESKTOP_DENIED".into(), denied.into());
            let result = read_with_environment("op://Development/test/secret", env);
            let trace = std::fs::read_to_string(dir.join("trace")).unwrap_or_default();
            std::fs::remove_file(dir.join("trace")).ok();
            assert_eq!(result.ok().as_deref(), expected, "{service_result}");
            assert_eq!(trace, expected_trace, "{service_result}");
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
