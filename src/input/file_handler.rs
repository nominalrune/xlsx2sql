// Input validation utilities
use std::path::Path;
use crate::errors::InputError;

pub fn validate_file_exists(path: &Path) -> Result<(), InputError> {
    if !path.exists() {
        return Err(InputError::FileNotFound(format!("{}", path.display())));
    }
    Ok(())
}

pub fn validate_file_format(path: &Path) -> Result<(), InputError> {
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        if ext != "xlsx" && ext != "xls" {
            return Err(InputError::InvalidFormat);
        }
    } else {
        return Err(InputError::InvalidFormat);
    }
    Ok(())
}
