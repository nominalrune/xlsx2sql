# xlsx2sql Test Plan

## Overview

This test plan provides comprehensive testing coverage for the xlsx2sql tool, ensuring reliability, performance, and correctness across all components and use cases. The plan is structured around the modular architecture defined in the design document.

## Test Strategy

### Testing Pyramid
```
                    ┌─────────────────┐
                    │ E2E/System Tests│
                    └─────────────────┘
                ┌─────────────────────────┐
                │  Integration Tests      │
                └─────────────────────────┘
        ┌─────────────────────────────────────┐
        │         Unit Tests                  │
        └─────────────────────────────────────┘
```

### Test Categories
1. **Unit Tests** (70%): Individual component testing
2. **Integration Tests** (20%): Component interaction testing
3. **End-to-End Tests** (10%): Full workflow testing

## Unit Tests

### 1. Data Model Tests (`src/parser/data_model.rs`)

#### 1.1 SqlValue Conversion Tests
```rust
#[cfg(test)]
mod sql_value_tests {
    use super::*;
    use calamine::DataType;

    #[test]
    fn test_datatype_empty_to_null() {
        assert_eq!(SqlValue::from(&DataType::Empty), SqlValue::Null);
    }

    #[test]
    fn test_datatype_string_conversion() {
        let text = "test string";
        let data_type = DataType::String(text.to_string());
        assert_eq!(SqlValue::from(&data_type), SqlValue::Text(text.to_string()));
    }

    #[test]
    fn test_datatype_numeric_conversions() {
        assert_eq!(SqlValue::from(&DataType::Float(3.14)), SqlValue::Number(3.14));
        assert_eq!(SqlValue::from(&DataType::Int(42)), SqlValue::Integer(42));
    }

    #[test]
    fn test_datatype_boolean_conversion() {
        assert_eq!(SqlValue::from(&DataType::Bool(true)), SqlValue::Boolean(true));
        assert_eq!(SqlValue::from(&DataType::Bool(false)), SqlValue::Boolean(false));
    }

    #[test]
    fn test_datatype_datetime_conversion() {
        let dt = chrono::NaiveDateTime::from_timestamp(1234567890, 0);
        let data_type = DataType::DateTime(dt.unwrap());
        match SqlValue::from(&data_type) {
            SqlValue::DateTime(_) => (),
            _ => panic!("Expected DateTime variant"),
        }
    }

    #[test]
    fn test_datatype_error_to_null() {
        let error_type = DataType::Error(calamine::CellErrorType::Div0);
        assert_eq!(SqlValue::from(&error_type), SqlValue::Null);
    }

    #[test]
    fn test_unicode_characters() {
        let japanese_text = "業務用";
        let chinese_text = "测试";
        let emoji_text = "😀🎉";
        
        for text in [japanese_text, chinese_text, emoji_text] {
            let data_type = DataType::String(text.to_string());
            match SqlValue::from(&data_type) {
                SqlValue::Text(s) => assert_eq!(s, text),
                _ => panic!("Expected Text variant for: {}", text),
            }
        }
    }
}
```

#### 1.2 WorkbookData Tests
```rust
#[test]
fn test_workbook_data_creation() {
    let workbook = WorkbookData { sheets: vec![] };
    assert!(workbook.sheets.is_empty());
}

#[test]
fn test_sheet_data_with_range() {
    // Test with mock Range<DataType>
    // Verify sheet name and range integration
}
```

### 2. Parser Layer Tests (`src/parser/`)

#### 2.1 CalamineXlsxParser Tests
```rust
#[cfg(test)]
mod parser_tests {
    use super::*;
    use std::path::Path;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_xlsx() {
        // Create test XLSX file with known data
        let test_file = create_test_xlsx();
        let parser = CalamineXlsxParser;
        
        let result = parser.parse(&test_file).unwrap();
        assert!(!result.sheets.is_empty());
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let parser = CalamineXlsxParser;
        let result = parser.parse(Path::new("nonexistent.xlsx"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_format() {
        // Create a non-Excel file with .xlsx extension
        let invalid_file = create_invalid_file();
        let parser = CalamineXlsxParser;
        
        let result = parser.parse(&invalid_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_sheets_parsing() {
        let test_file = create_multi_sheet_xlsx();
        let parser = CalamineXlsxParser;
        
        let result = parser.parse(&test_file).unwrap();
        assert_eq!(result.sheets.len(), 2);
        assert_eq!(result.sheets[0].name, "businesses");
        assert_eq!(result.sheets[1].name, "cards");
    }

    #[test]
    fn test_empty_sheet_handling() {
        let test_file = create_xlsx_with_empty_sheet();
        let parser = CalamineXlsxParser;
        
        let result = parser.parse(&test_file).unwrap();
        // Should skip empty sheets or handle gracefully
    }
}
```

#### 2.2 SheetData Methods Tests
```rust
#[test]
fn test_get_columns_from_first_row() {
    let sheet_data = create_test_sheet_data();
    let columns = sheet_data.get_columns().unwrap();
    
    assert_eq!(columns, vec!["id", "name", "status"]);
}

#[test]
fn test_get_columns_empty_sheet() {
    let empty_sheet = create_empty_sheet_data();
    let result = empty_sheet.get_columns();
    
    assert!(matches!(result, Err(ParseError::EmptySheet)));
}

#[test]
fn test_get_data_rows_skips_header() {
    let sheet_data = create_test_sheet_data();
    let rows: Vec<_> = sheet_data.get_data_rows().collect();
    
    // Should skip the first row (header)
    assert_eq!(rows.len(), 2); // Assuming 3 total rows, 1 header + 2 data
}

#[test]
fn test_column_names_with_special_characters() {
    let sheet_data = create_sheet_with_special_columns();
    let columns = sheet_data.get_columns().unwrap();
    
    assert!(columns.contains(&"X_ユーザーID?".to_string()));
    assert!(columns.contains(&"enterprise_number".to_string()));
}
```

### 3. Generator Layer Tests (`src/generator/`)

#### 3.1 MySqlGenerator Tests
```rust
#[cfg(test)]
mod generator_tests {
    use super::*;

    #[test]
    fn test_generate_single_sheet() {
        let workbook_data = create_test_workbook_data();
        let generator = MySqlGenerator;
        
        let statements = generator.generate(&workbook_data).unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].table_name, "businesses");
    }

    #[test]
    fn test_generate_multiple_sheets() {
        let workbook_data = create_multi_sheet_workbook_data();
        let generator = MySqlGenerator;
        
        let statements = generator.generate(&workbook_data).unwrap();
        assert_eq!(statements.len(), 2);
        
        let table_names: Vec<_> = statements.iter().map(|s| &s.table_name).collect();
        assert!(table_names.contains(&&"businesses".to_string()));
        assert!(table_names.contains(&&"cards".to_string()));
    }

    #[test]
    fn test_generate_empty_sheet_skipped() {
        let workbook_data = create_workbook_with_empty_sheet();
        let generator = MySqlGenerator;
        
        let statements = generator.generate(&workbook_data).unwrap();
        // Empty sheets should be skipped
        assert!(statements.is_empty() || statements.iter().all(|s| !s.values.is_empty()));
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
    fn test_format_sql_value_escaping() {
        let generator = MySqlGenerator;
        
        assert_eq!(generator.format_sql_value(&SqlValue::Null), "NULL");
        assert_eq!(generator.format_sql_value(&SqlValue::Integer(42)), "42");
        assert_eq!(generator.format_sql_value(&SqlValue::Number(3.14)), "3.14");
        assert_eq!(generator.format_sql_value(&SqlValue::Boolean(true)), "1");
        assert_eq!(generator.format_sql_value(&SqlValue::Boolean(false)), "0");
        assert_eq!(generator.format_sql_value(&SqlValue::Text("test".to_string())), "'test'");
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
```

### 4. Input Layer Tests (`src/input/`)

#### 4.1 FileInputHandler Tests
```rust
#[cfg(test)]
mod input_tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_file_path_valid() {
        let handler = FileInputHandler;
        // Test with valid file path
    }

    #[test]
    fn test_read_file_success() {
        let temp_file = NamedTempFile::new().unwrap();
        let handler = FileInputHandler;
        
        let result = handler.read_file(temp_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_file_not_found() {
        let handler = FileInputHandler;
        let result = handler.read_file(Path::new("nonexistent.xlsx"));
        
        assert!(matches!(result, Err(InputError::FileNotFound(_))));
    }

    #[test]
    fn test_read_file_permission_denied() {
        // Create file with no read permissions
        // Test permission denied scenario
    }
}
```

### 5. Output Layer Tests (`src/output/`)

#### 5.1 OutputWriter Tests
```rust
#[cfg(test)]
mod output_tests {
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
        
        // Test stdout writing (may need to capture output)
        let result = writer.write(content, &OutputDestination::Stdout);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_file_permission_error() {
        let writer = FileOutputWriter;
        let content = "test content";
        
        // Try to write to a directory without write permissions
        let result = writer.write(content, &OutputDestination::File(PathBuf::from("/root/test.sql")));
        assert!(result.is_err());
    }
}
```

### 6. Error Handling Tests

#### 6.1 Error Conversion Tests
```rust
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_calamine_error_conversion() {
        let calamine_error = calamine::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound, 
            "File not found"
        ));
        let xlsx_error: Xlsx2SqlError = calamine_error.into();
        
        assert!(matches!(xlsx_error, Xlsx2SqlError::Calamine(_)));
    }

    #[test]
    fn test_parse_error_conversion() {
        let parse_error = ParseError::EmptySheet;
        let xlsx_error: Xlsx2SqlError = parse_error.into();
        
        assert!(matches!(xlsx_error, Xlsx2SqlError::Parse(_)));
    }

    #[test]
    fn test_error_display() {
        let error = Xlsx2SqlError::Parse(ParseError::EmptySheet);
        let error_string = format!("{}", error);
        
        assert!(error_string.contains("Parse error"));
        assert!(error_string.contains("Sheet has no data"));
    }
}
```

## Integration Tests

### 1. Parser-Generator Integration (`tests/integration/`)

#### 1.1 End-to-End Processing Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_xlsx_to_sql_complete_flow() {
        // Create test XLSX file based on spec example
        let test_file = create_spec_example_xlsx();
        let parser = CalamineXlsxParser;
        let generator = MySqlGenerator;
        
        // Parse
        let workbook_data = parser.parse(&test_file).unwrap();
        
        // Generate
        let statements = generator.generate(&workbook_data).unwrap();
        
        // Verify results match specification
        assert_eq!(statements.len(), 2); // businesses and cards tables
        
        // Verify businesses table
        let businesses_stmt = statements.iter().find(|s| s.table_name == "businesses").unwrap();
        assert!(businesses_stmt.columns.contains(&"id".to_string()));
        assert!(businesses_stmt.columns.contains(&"enterprise_number".to_string()));
        
        // Verify cards table
        let cards_stmt = statements.iter().find(|s| s.table_name == "cards").unwrap();
        assert!(cards_stmt.columns.contains(&"business_id".to_string()));
        assert!(cards_stmt.columns.contains(&"name".to_string()));
    }

    #[test]
    fn test_multiple_format_support() {
        for format in ["xlsx", "xls", "xlsm", "ods"] {
            let test_file = create_test_file_with_format(format);
            let parser = CalamineXlsxParser;
            
            let result = parser.parse(&test_file);
            assert!(result.is_ok(), "Failed to parse {} format", format);
        }
    }

    #[test]
    fn test_large_file_processing() {
        // Create large test file (1000+ rows)
        let large_file = create_large_test_file(1000);
        let parser = CalamineXlsxParser;
        let generator = MySqlGenerator;
        
        let start_time = std::time::Instant::now();
        
        let workbook_data = parser.parse(&large_file).unwrap();
        let statements = generator.generate(&workbook_data).unwrap();
        
        let duration = start_time.elapsed();
        
        // Verify performance (should be fast based on calamine benchmarks)
        assert!(duration.as_secs() < 5, "Processing took too long: {:?}", duration);
        assert!(!statements.is_empty());
    }
}
```

### 2. CLI Integration Tests

#### 2.1 Command Line Interface Tests
```rust
#[cfg(test)]
mod cli_integration_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cli_file_argument() {
        let test_file = create_test_xlsx();
        
        let mut cmd = Command::cargo_bin("xlsx2sql").unwrap();
        cmd.arg(test_file.path());
        
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("INSERT INTO"));
    }

    #[test]
    fn test_cli_file_option() {
        let test_file = create_test_xlsx();
        
        let mut cmd = Command::cargo_bin("xlsx2sql").unwrap();
        cmd.arg("-f").arg(test_file.path());
        
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("INSERT INTO"));
    }

    #[test]
    fn test_cli_output_file() {
        let test_file = create_test_xlsx();
        let output_file = NamedTempFile::new().unwrap();
        
        let mut cmd = Command::cargo_bin("xlsx2sql").unwrap();
        cmd.arg("-f").arg(test_file.path())
           .arg("-o").arg(output_file.path());
        
        cmd.assert().success();
        
        // Verify output file content
        let content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("INSERT INTO"));
    }

    #[test]
    fn test_cli_nonexistent_file() {
        let mut cmd = Command::cargo_bin("xlsx2sql").unwrap();
        cmd.arg("nonexistent.xlsx");
        
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("File not found"));
    }

    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("xlsx2sql").unwrap();
        cmd.arg("--help");
        
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("xlsx2sql"))
            .stdout(predicate::str::contains("Convert xlsx to sql"));
    }

    #[test]
    fn test_cli_version() {
        let mut cmd = Command::cargo_bin("xlsx2sql").unwrap();
        cmd.arg("--version");
        
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("xlsx2sql"));
    }
}
```

## End-to-End Tests

### 1. Specification Compliance Tests

#### 1.1 Spec Example Validation
```rust
#[cfg(test)]
mod e2e_spec_tests {
    use super::*;

    #[test]
    fn test_spec_example_exact_match() {
        // Create XLSX file exactly matching the specification example
        let spec_xlsx = create_exact_spec_example();
        
        // Process through complete pipeline
        let output = process_file_to_string(&spec_xlsx).unwrap();
        
        // Verify output matches expected SQL from spec
        assert!(output.contains("INSERT INTO `businesses`"));
        assert!(output.contains("INSERT INTO `cards`"));
        
        // Verify specific data values from spec
        assert!(output.contains("9700001"));
        assert!(output.contains("97businessid001"));
        assert!(output.contains("業務用"));
        assert!(output.contains("日常用"));
        
        // Verify Unicode handling
        assert!(output.contains("97userid001"));
        assert!(output.contains("97userid002"));
    }

    #[test]
    fn test_japanese_character_preservation() {
        let japanese_xlsx = create_japanese_test_file();
        let output = process_file_to_string(&japanese_xlsx).unwrap();
        
        // Verify all Japanese characters are preserved correctly
        assert!(output.contains("業務用"));
        assert!(output.contains("日常用"));
        assert!(output.contains("ユーザーID"));
    }

    #[test]
    fn test_special_characters_in_column_names() {
        let special_char_xlsx = create_special_char_columns_file();
        let output = process_file_to_string(&special_char_xlsx).unwrap();
        
        // Verify column names with special characters are handled
        assert!(output.contains("`X_ユーザーID?`"));
        assert!(output.contains("`enterprise_number`"));
    }
}
```

### 2. Real-world Scenario Tests

#### 2.1 Various File Scenarios
```rust
#[test]
fn test_mixed_data_types() {
    let mixed_file = create_mixed_datatype_file();
    let output = process_file_to_string(&mixed_file).unwrap();
    
    // Should handle strings, numbers, dates, booleans, and empty cells
    assert!(output.contains("NULL")); // Empty cells
    assert!(output.contains("'text'")); // Strings
    assert!(output.contains("42")); // Integers
    assert!(output.contains("3.14")); // Floats
    assert!(output.contains("1")); // Boolean true
    assert!(output.contains("0")); // Boolean false
}

#[test]
fn test_edge_cases() {
    // Test various edge cases
    test_single_column_sheet();
    test_single_row_sheet();
    test_empty_cells_only();
    test_very_long_text();
    test_special_sql_characters();
}

#[test]
fn test_performance_large_dataset() {
    // Test with dataset similar to calamine benchmark (1M rows)
    let large_dataset = create_performance_test_file(1_000_000, 41);
    
    let start_time = std::time::Instant::now();
    let output = process_file_to_string(&large_dataset).unwrap();
    let duration = start_time.elapsed();
    
    // Should complete in reasonable time (based on calamine's 25s for 1M rows)
    assert!(duration.as_secs() < 30, "Processing too slow: {:?}", duration);
    assert!(!output.is_empty());
}
```

## Test Data Management

### 1. Test File Creation Utilities

```rust
// Utility functions for creating test files

fn create_spec_example_xlsx() -> NamedTempFile {
    // Create XLSX file matching the exact specification example
    // businesses sheet with specified columns and data
    // cards sheet with business_id and name
}

fn create_test_xlsx() -> NamedTempFile {
    // Create simple test XLSX for basic functionality
}

fn create_multi_sheet_xlsx() -> NamedTempFile {
    // Create XLSX with multiple sheets
}

fn create_large_test_file(rows: usize) -> NamedTempFile {
    // Create large file for performance testing
}

fn create_japanese_test_file() -> NamedTempFile {
    // Create file with extensive Japanese character usage
}

fn create_mixed_datatype_file() -> NamedTempFile {
    // Create file with all supported data types
}
```

### 2. Test Data Organization

```
tests/
├── data/
│   ├── spec_example.xlsx          # Exact spec example
│   ├── simple_test.xlsx          # Basic test case
│   ├── multi_sheet.xlsx          # Multiple sheets
│   ├── japanese_chars.xlsx       # Unicode testing
│   ├── mixed_types.xlsx          # All data types
│   ├── edge_cases/
│   │   ├── empty_sheet.xlsx
│   │   ├── single_column.xlsx
│   │   ├── single_row.xlsx
│   │   └── special_chars.xlsx
│   └── performance/
│       ├── medium_1k.xlsx        # 1K rows
│       ├── large_10k.xlsx        # 10K rows
│       └── xlarge_100k.xlsx      # 100K rows
├── integration/
│   ├── parser_generator.rs
│   └── cli_tests.rs
├── e2e/
│   ├── spec_compliance.rs
│   └── real_world_scenarios.rs
└── common/
    ├── test_utils.rs
    └── file_creators.rs
```

## Continuous Integration Tests

### 1. Platform Compatibility Tests

```yaml
# .github/workflows/test.yml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macOS-latest]
    rust: [stable, beta, nightly]

test_commands:
  - cargo test --all-features
  - cargo test --release
  - cargo test integration_tests
  - cargo test e2e_tests
```

### 2. Format Compatibility Tests

```rust
#[test]
fn test_all_supported_formats() {
    for format in ["xlsx", "xls", "xlsm", "xlsb", "ods"] {
        let test_file = create_format_specific_file(format);
        let result = process_file_to_string(&test_file);
        
        assert!(result.is_ok(), "Failed to process {} format: {:?}", format, result.err());
    }
}
```

## Test Execution Plan

### 1. Development Testing
- **Unit tests**: Run on every code change
- **Integration tests**: Run on pull requests

### 2. Pre-release Testing
- **Full test suite**: All unit, integration, and E2E tests
- **Format compatibility**: Test all supported formats
- **Platform testing**: Test on all target platforms

### 3. Release Testing
- **Specification compliance**: Verify exact spec adherence
- **Real-world scenarios**: Test with actual Excel files
- **Documentation examples**: Verify all examples work

## Test Metrics and Coverage

### 1. Coverage Targets
- **Unit tests**: 90%+ code coverage
- **Integration tests**: 80%+ integration coverage
- **E2E tests**: 100% user workflow coverage

### 2. Quality Gates
- All tests must pass before merge
- Memory usage within acceptable limits
- No security vulnerabilities in dependencies

### 3. Test Reporting
- Coverage reports for each PR
- Test execution time monitoring
- Failure analysis and trends

## Conclusion

This comprehensive test plan ensures the xlsx2sql tool meets all requirements from the specification while maintaining high quality, performance, and reliability. The testing strategy covers all components, integration points, and real-world usage scenarios, providing confidence in the tool's correctness and robustness.

Key testing priorities:
1. **Specification compliance**: Exact adherence to xlsx2sql requirements
2. **Unicode support**: Proper handling of Japanese and other international characters
3. **Error handling**: Robust error scenarios and recovery
4. **Format support**: All calamine-supported file formats
5. **Security**: SQL injection prevention and input validation
