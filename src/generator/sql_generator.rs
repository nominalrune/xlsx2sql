use crate::errors::GeneratorError;
use crate::generator::formatter::SqlFormatter;
use crate::parser::data_model::{SqlStatement, SqlValue, WorkbookData};

pub trait SqlGenerator {
    fn generate(&self, data: &WorkbookData) -> Result<Vec<SqlStatement>, GeneratorError>;
    fn format_statement(&self, statement: &SqlStatement) -> String;
}

pub struct MySqlGenerator;

impl SqlGenerator for MySqlGenerator {
    fn generate(&self, data: &WorkbookData) -> Result<Vec<SqlStatement>, GeneratorError> {
        let mut statements = Vec::new();

        for sheet in &data.sheets {
            let columns = sheet.get_columns()?;
            if columns.is_empty() {
                continue;
            }

            let mut values = Vec::new();
            for row in sheet.get_data_rows() {
                let row_values: Vec<SqlValue> = row.iter().map(SqlValue::from).collect();
                values.push(row_values);
            }

            if !values.is_empty() {
                statements.push(SqlStatement {
                    table_name: sheet.name.clone(),
                    columns,
                    values,
                });
            }
        }

        // Check if no data was found
        if statements.is_empty() {
            return Err(GeneratorError::NoData);
        }

        Ok(statements)
    }

    fn format_statement(&self, statement: &SqlStatement) -> String {
        let table_name = SqlFormatter::format_identifier(&statement.table_name);
        let columns = statement
            .columns
            .iter()
            .map(|col| SqlFormatter::format_identifier(col))
            .collect::<Vec<_>>()
            .join(", ");

        let values_str = statement
            .values
            .iter()
            .map(|row| {
                let row_str = row
                    .iter()
                    .map(|val| self.format_sql_value(val))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("({row_str})")
            })
            .collect::<Vec<_>>()
            .join(",\n");

        format!("INSERT INTO {table_name} ({columns}) VALUES\n{values_str};")
    }
}

impl MySqlGenerator {
    fn format_sql_value(&self, value: &SqlValue) -> String {
        match value {
            SqlValue::Null => "NULL".to_string(),
            SqlValue::Text(s) => SqlFormatter::format_string_literal(s),
            SqlValue::Number(f) => f.to_string(),
            SqlValue::Integer(i) => i.to_string(),
            SqlValue::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
            SqlValue::DateTime(dt) => format!("'{dt}'"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::data_model::SqlValue;

    #[test]
    fn test_format_sql_value_escaping() {
        let generator = MySqlGenerator;

        assert_eq!(generator.format_sql_value(&SqlValue::Null), "NULL");
        assert_eq!(generator.format_sql_value(&SqlValue::Integer(42)), "42");
        assert_eq!(generator.format_sql_value(&SqlValue::Number(3.14)), "3.14");
        assert_eq!(generator.format_sql_value(&SqlValue::Boolean(true)), "1");
        assert_eq!(generator.format_sql_value(&SqlValue::Boolean(false)), "0");
        assert_eq!(
            generator.format_sql_value(&SqlValue::Text("test".to_string())),
            "'test'"
        );
    }

    #[test]
    fn test_sql_injection_prevention() {
        let generator = MySqlGenerator;
        let malicious_text = "'; DROP TABLE users; --";
        let sql_value = SqlValue::Text(malicious_text.to_string());

        let formatted = generator.format_sql_value(&sql_value);
        assert_eq!(formatted, "'''; DROP TABLE users; --'");
    }

    #[test]
    fn test_unicode_in_sql_generation() {
        let generator = MySqlGenerator;
        let japanese_text = "業務用";
        let sql_value = SqlValue::Text(japanese_text.to_string());

        let formatted = generator.format_sql_value(&sql_value);
        assert_eq!(formatted, format!("'{}'", japanese_text));
    }

    #[test]
    fn test_format_statement_basic() {
        let statement = SqlStatement {
            table_name: "test_table".to_string(),
            columns: vec!["id".to_string(), "name".to_string()],
            values: vec![
                vec![SqlValue::Integer(1), SqlValue::Text("John".to_string())],
                vec![SqlValue::Integer(2), SqlValue::Text("Jane".to_string())],
            ],
        };

        let generator = MySqlGenerator;
        let sql = generator.format_statement(&statement);

        assert!(sql.contains("INSERT INTO `test_table`"));
        assert!(sql.contains("(`id`, `name`)"));
        assert!(sql.contains("VALUES"));
        assert!(sql.contains("(1,'John')"));
        assert!(sql.contains("(2,'Jane')"));
    }

    #[test]
    fn test_table_name_sanitization() {
        let statement = SqlStatement {
            table_name: "table with spaces".to_string(),
            columns: vec!["id".to_string()],
            values: vec![vec![SqlValue::Integer(1)]],
        };

        let generator = MySqlGenerator;
        let sql = generator.format_statement(&statement);

        assert!(sql.contains("`table with spaces`"));
    }
}
