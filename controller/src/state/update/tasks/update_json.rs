use serde_json::{error::Error as SerdeError, Value};
use std::fs;

#[derive(Debug, thiserror::Error)]
pub enum UpdateJsonError {
    #[error("Failed to read JSON file '{0}': {1}")]
    FileReadError(String, #[source] std::io::Error),
    #[error("Failed to parse JSON in file '{0}': {1}")]
    JsonParseError(String, #[source] SerdeError),
    #[error("Invalid field path '{0}'")]
    InvalidFieldPath(String),
    #[error("Failed to write JSON file '{0}': {1}")]
    FileWriteError(String, #[source] std::io::Error),
}

pub fn run(file: &String, field: &String, value: &String) -> Result<(), UpdateJsonError> {
    // Array updates are not supported.
    // It's unclear whether the operation is an addition or a completely new write.
    log::info!(
        "Updating JSON file '{}' field '{}' with value '{}'",
        file,
        field,
        value
    );

    let file_content = fs::read_to_string(file)
        .map_err(|e| UpdateJsonError::FileReadError(file.to_string(), e))?;

    let mut json_data: Value = serde_json::from_str(&file_content)
        .map_err(|e| UpdateJsonError::JsonParseError(file.to_string(), e))?;

    let parts: Vec<&str> = field.split('.').collect();
    let mut current = &mut json_data;
    for part in &parts[..parts.len() - 1] {
        current = current
            .get_mut(part)
            .ok_or_else(|| UpdateJsonError::InvalidFieldPath(field.to_string()))?;
    }

    current[parts.last().unwrap()] = Value::String(value.to_string());

    fs::write(
        file,
        serde_json::to_string_pretty(&json_data)
            .map_err(|e| UpdateJsonError::JsonParseError(file.to_string(), e))?,
    )
    .map_err(|e| UpdateJsonError::FileWriteError(file.to_string(), e))?;

    Ok(())
}
