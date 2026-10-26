use std::{os::unix::fs::PermissionsExt, path::Path, process::Command};

fn tool(root: &Path, name: &str, script: &str) {
    let path = root.join(name);
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn operational_failures_are_json_and_report_published_roots() {
    let root = std::env::temp_dir().join(format!("boilerplate-json-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    for (case, expected_stage, published) in [
        ("missing-git", "validation", false),
        ("broken-git", "git", true),
        ("missing-uv", "install", true),
    ] {
        let tools = root.join(format!("tools-{case}"));
        std::fs::create_dir(&tools).unwrap();
        if case == "broken-git" {
            tool(
                &tools,
                "git",
                "#!/bin/sh\nif [ \"$1\" = --version ]; then exit 0; fi\necho external-log\nexit 1\n",
            );
        }
        let destination = root.join(case);
        let mut command = Command::new(env!("CARGO_BIN_EXE_boilerplate"));
        command
            .env("PATH", &tools)
            .env("HOME", &root)
            .env("SECRET_TEST", "must-not-appear");
        command
            .args([
                "new",
                case,
                "--type",
                "backend",
                "--backend",
                "python-fastapi",
                "--non-interactive",
                "--json",
                "--github",
                "no",
                "--destination",
            ])
            .arg(&destination);
        command.args([
            "--git",
            if case == "missing-uv" { "no" } else { "yes" },
            "--install",
            if case == "missing-uv" { "yes" } else { "no" },
        ]);
        let output = command.output().unwrap();
        assert!(!output.status.success());
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["ok"], false);
        assert_eq!(json["error"]["stage"], expected_stage);
        assert_eq!(
            json["created"].as_array().unwrap().len(),
            usize::from(published)
        );
        assert_eq!(destination.join("pyproject.toml").exists(), published);
        assert!(!String::from_utf8_lossy(&output.stdout).contains("must-not-appear"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("external-log"));
    }
    std::fs::remove_dir_all(root).unwrap();
}
