use calamine::{open_workbook_auto, Reader};
use std::path::Path;
use crate::errors::ParseError;
use crate::parser::data_model::{WorkbookData, SheetData};

pub trait XlsxParser {
    fn parse(&self, file_path: &Path) -> Result<WorkbookData, ParseError>;
}

pub struct CalamineXlsxParser;

impl XlsxParser for CalamineXlsxParser {
    fn parse(&self, file_path: &Path) -> Result<WorkbookData, ParseError> {
        let mut workbook = open_workbook_auto(file_path)?;
        let sheet_names = workbook.sheet_names().to_owned();
        
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
