use std::path::{Path, PathBuf};
use crate::errors::InputError;

pub trait InputHandler {
    fn get_file_path(&self) -> Result<PathBuf, InputError>;
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, InputError>;
}

pub struct FileInputHandler {
    file_path: Option<PathBuf>,
}

impl FileInputHandler {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        Self { file_path }
    }
}

impl InputHandler for FileInputHandler {
    fn get_file_path(&self) -> Result<PathBuf, InputError> {
        match &self.file_path {
            Some(path) => Ok(path.clone()),
            None => Err(InputError::FileNotFound("No file path provided".to_string())),
        }
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, InputError> {
        if !path.exists() {
            return Err(InputError::FileNotFound(format!("{}", path.display())));
        }
        
        std::fs::read(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => InputError::FileNotFound(format!("{}", path.display())),
            std::io::ErrorKind::PermissionDenied => InputError::PermissionDenied(format!("{}", path.display())),
            _ => InputError::Io(e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_read_file_not_found() {
        let handler = FileInputHandler::new(None);
        let result = handler.read_file(Path::new("nonexistent.xlsx"));
        
        assert!(matches!(result, Err(InputError::FileNotFound(_))));
    }

    #[test]
    fn test_get_file_path_none() {
        let handler = FileInputHandler::new(None);
        let result = handler.get_file_path();
        
        assert!(matches!(result, Err(InputError::FileNotFound(_))));
    }

    #[test]
    fn test_get_file_path_some() {
        let path = PathBuf::from("test.xlsx");
        let handler = FileInputHandler::new(Some(path.clone()));
        let result = handler.get_file_path().unwrap();
        
        assert_eq!(result, path);
    }
}
