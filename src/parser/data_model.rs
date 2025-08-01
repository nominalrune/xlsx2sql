use calamine::{Data, Range};
use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct WorkbookData {
    pub sheets: Vec<SheetData>,
}

#[derive(Debug, Clone)]
pub struct SheetData {
    pub name: String,
    pub range: Range<Data>,
}

#[derive(Debug)]
pub struct SqlStatement {
    pub table_name: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<SqlValue>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqlValue {
    Text(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    DateTime(String),
    Null,
}

impl From<&Data> for SqlValue {
    fn from(data: &Data) -> Self {
        match data {
            Data::Empty => SqlValue::Null,
            Data::String(s) => SqlValue::Text(s.clone()),
            Data::Float(f) => SqlValue::Number(*f),
            Data::Int(i) => SqlValue::Integer(*i),
            Data::Bool(b) => SqlValue::Boolean(*b),
            Data::DateTime(dt) => {
                // Convert Excel datetime to SQL datetime format
                // Excel dates start from 1900-01-01 (serial 1)
                let excel_epoch = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap(); // Excel epoch is 1899-12-30
                let days = dt.as_f64() as i64;
                let seconds = ((dt.as_f64() - days as f64) * 86400.0) as u32;

                if let Some(date) = excel_epoch.checked_add_signed(chrono::Duration::days(days)) {
                    if let Some(datetime) = date.and_hms_opt(0, 0, 0).and_then(|dt| {
                        dt.checked_add_signed(chrono::Duration::seconds(seconds as i64))
                    }) {
                        SqlValue::DateTime(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
                    } else {
                        SqlValue::Number(dt.as_f64())
                    }
                } else {
                    SqlValue::Number(dt.as_f64())
                }
            }
            Data::Error(_) => SqlValue::Null,
            Data::DateTimeIso(dt) => SqlValue::DateTime(dt.clone()),
            Data::DurationIso(dur) => SqlValue::Text(dur.clone()),
        }
    }
}

impl SheetData {
    pub fn get_columns(&self) -> Result<Vec<String>, crate::errors::ParseError> {
        if let Some(first_row) = self.range.rows().next() {
            let columns: Vec<String> = first_row
                .iter()
                .map(|cell| match cell {
                    Data::String(s) => s.clone(),
                    Data::Empty => String::new(),
                    other => format!("{}", other),
                })
                .collect();

            // Check if all headers are empty (missing headers)
            if columns.iter().all(|col| col.trim().is_empty()) {
                return Err(crate::errors::ParseError::MissingHeaders);
            }

            Ok(columns)
        } else {
            Err(crate::errors::ParseError::EmptySheet)
        }
    }

    pub fn get_data_rows(&self) -> impl Iterator<Item = &[Data]> {
        self.range.rows().skip(1) // Skip header row
    }
}

// Tests will be added later with proper test data
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workbook_data_creation() {
        let workbook = WorkbookData { sheets: vec![] };
        assert!(workbook.sheets.is_empty());
    }
}
