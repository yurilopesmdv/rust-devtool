use std::error::Error;
use std::fs;

pub fn format_json(file_path: &str) -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;

    let json: serde_json::Value = serde_json::from_str(&content)?;

    let formatted = serde_json::to_string_pretty(&json)?;

    Ok(formatted)
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn should_return_error_when_file_does_not_exist() {
        let result = format_json("non_existent_file.json");
        assert!(result.is_err());
    }

    #[test]
    fn should_return_valid_json() {
        let path = std::env::temp_dir().join(format!("devtool-test-{}.json", std::process::id()));

        std::fs::write(&path, r#"{"name":"DevTool"}"#).unwrap();

        let result = format_json(path.to_str().unwrap());

        std::fs::remove_file(&path).unwrap();

        let formated = result.unwrap();

        assert!(formated.contains("\"name\": \"DevTool\""));
    }
}
