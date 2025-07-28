use crate::errors::ParseError;
use crate::input::file_handler::{validate_file_exists, validate_file_format};
use crate::parser::data_model::{SheetData, WorkbookData};
use calamine::{open_workbook_auto, Reader};
use std::path::Path;

pub trait XlsxParser {
    fn parse(&self, file_path: &Path) -> Result<WorkbookData, ParseError>;
}

pub struct CalamineXlsxParser;

impl XlsxParser for CalamineXlsxParser {
    fn parse(&self, file_path: &Path) -> Result<WorkbookData, ParseError> {
        // Validate file exists and format
        validate_file_exists(file_path).map_err(|_| ParseError::InvalidFormat)?;
        validate_file_format(file_path).map_err(|_| ParseError::InvalidFormat)?;

        let mut workbook = open_workbook_auto(file_path)?;
        let sheet_names = workbook.sheet_names().to_owned();

        if sheet_names.is_empty() {
            return Err(ParseError::InvalidFormat);
        }

        let mut sheets = Vec::new();
        for sheet_name in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                sheets.push(SheetData {
                    name: sheet_name,
                    range,
                });
            }
        }

        Ok(WorkbookData { sheets })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse_nonexistent_file() {
        let parser = CalamineXlsxParser;
        let result = parser.parse(Path::new("nonexistent.xlsx"));
        assert!(result.is_err());
    }
}
