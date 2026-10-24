use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;

pub fn calculate_hash(file_path: &str) -> Result<String, Box<dyn Error>> {
    let content = fs::read(file_path)?;

    let mut hasher = Sha256::new();

    hasher.update(&content);

    let result = hasher.finalize();

    let hash: String = result.iter().map(|byte| format!("{:02x}", byte)).collect();

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_error_when_file_does_not_exist() {
        let result = calculate_hash("arquivo_inexistente_123.txt");

        assert!(result.is_err());
    }

    #[test]
    fn should_calculate_hash_correctly() {
        let file_path = "teste_hash.txt";

        std::fs::write(file_path, "hello").unwrap();

        let result = calculate_hash(file_path).unwrap();

        assert_eq!(
            result,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );

        std::fs::remove_file(file_path).unwrap();
    }
}
