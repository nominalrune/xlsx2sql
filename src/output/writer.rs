use std::path::PathBuf;
use crate::errors::OutputError;

pub trait OutputWriter {
    fn write(&self, content: &str, destination: &OutputDestination) -> Result<(), OutputError>;
}

#[derive(Debug)]
pub enum OutputDestination {
    File(PathBuf),
    Stdout,
}

pub struct FileOutputWriter;

impl OutputWriter for FileOutputWriter {
    fn write(&self, content: &str, destination: &OutputDestination) -> Result<(), OutputError> {
        match destination {
            OutputDestination::File(path) => {
                std::fs::write(path, content).map_err(OutputError::Io)?;
            }
            OutputDestination::Stdout => {
                println!("{}", content);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_to_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let writer = FileOutputWriter;
        let content = "INSERT INTO test VALUES (1, 'test');";
        
        let result = writer.write(content, &OutputDestination::File(temp_file.path().to_path_buf()));
        assert!(result.is_ok());
        
        // Verify file content
        let written_content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_write_to_stdout() {
        let writer = FileOutputWriter;
        let content = "INSERT INTO test VALUES (1, 'test');";
        
        // Test stdout writing (output goes to stdout)
        let result = writer.write(content, &OutputDestination::Stdout);
        assert!(result.is_ok());
    }
}
