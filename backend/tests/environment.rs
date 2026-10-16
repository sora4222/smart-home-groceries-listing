//! Configuration-source contract, independent of HTTP and database startup.
use grocery_backend::environment::load_environment;
use std::collections::HashMap;

#[test]
fn exported_literals_override_references_without_calling_a_provider() {
    let file = std::env::temp_dir().join(format!("grocery-{}.env", uuid::Uuid::new_v4()));
    std::fs::write(
        &file,
        "DATABASE_URL=op://Development/db/url\nCLERK_JWKS_URL='https://clerk.test/jwks'\n",
    )
    .unwrap();
    let inherited = HashMap::from([("DATABASE_URL".into(), "postgres://test".into())]);
    let result = load_environment(
        Some(&file),
        &inherited,
        &["DATABASE_URL", "CLERK_JWKS_URL"],
        |_| panic!("provider unnecessary"),
    )
    .unwrap();
    std::fs::remove_file(file).unwrap();
    assert_eq!(result["DATABASE_URL"], "postgres://test");
    assert_eq!(result["CLERK_JWKS_URL"], "https://clerk.test/jwks");
}

#[test]
fn every_supported_secret_can_use_a_reference_without_loading_unrelated_keys() {
    let keys = [
        "DATABASE_URL",
        "POSTGRES_USER",
        "POSTGRES_PASSWORD",
        "CLERK_SECRET_KEY",
        "CLERK_PUBLISHABLE_KEY",
        "VITE_CLERK_PUBLISHABLE_KEY",
        "CLERK_JWKS_URL",
        "ALEXA_SKILL_ID",
    ];
    let mut inherited: HashMap<String, String> = keys
        .iter()
        .map(|key| (key.to_string(), format!("op://Development/test/{key}")))
        .collect();
    inherited.insert("UNRELATED".into(), "op://Development/missing/item".into());
    let values = load_environment(None, &inherited, &keys, |reference| {
        Ok(format!(
            "resolved-{}",
            reference.rsplit('/').next().unwrap()
        ))
    })
    .unwrap();
    for key in keys {
        assert_eq!(values[key], format!("resolved-{key}"));
    }
    assert!(!values.contains_key("UNRELATED"));
}

#[test]
fn provider_errors_are_redacted_and_unresolved_values_fail_closed() {
    let inherited = HashMap::from([("DATABASE_URL".into(), "op://private/db/password".into())]);
    let error = load_environment(None, &inherited, &["DATABASE_URL"], |_| {
        Err(anyhow::anyhow!("provider output containing secret"))
    })
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "Cannot load DATABASE_URL from 1Password; check CLI access and the reference"
    );
    assert!(load_environment(None, &inherited, &["DATABASE_URL"], |_| Ok(String::new())).is_err());
}

#[test]
fn shared_source_selection_contract() {
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../scripts/environment-contract.json")).unwrap();
    for fixture in fixtures {
        let dir = std::env::temp_dir().join(format!("grocery-contract-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let file = dir.join(".env");
        if let Some(contents) = fixture["contents"].as_str() {
            std::fs::write(&file, contents).unwrap();
        }
        let selected = match fixture["source"].as_str().unwrap() {
            "default" => None,
            "empty" => Some(std::path::Path::new("")),
            "directory" => Some(dir.as_path()),
            _ => Some(file.as_path()),
        };
        let inherited = fixture["inherited"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.as_str().unwrap().to_string()))
            .collect();
        let result = load_environment(selected, &inherited, &["ALEXA_SKILL_ID"], |_| {
            panic!("provider must not run")
        });
        std::fs::remove_dir_all(dir).unwrap();
        if fixture["error"] == true {
            assert_eq!(
                result.unwrap_err().to_string(),
                "Cannot read environment file"
            );
        } else {
            assert_eq!(
                result.unwrap()["ALEXA_SKILL_ID"],
                fixture["expected"].as_str().unwrap()
            );
        }
    }
}
