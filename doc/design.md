# xlsx2sql Design Document

## Overview

xlsx2sql is a command-line tool written in Rust that converts Excel (.xlsx) files to SQL INSERT statements. The tool assumes that sheet names correspond to table names, the first row contains column names, and subsequent rows contain data records.

## Architecture

### High-Level Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Input Layer   │───▶│  Parser Layer   │───▶│ Generator Layer │───▶│  Output Layer   │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
```

1. **Input Layer**: Handles file input (drag-and-drop, `-f` option, stdin)
2. **Parser Layer**: Parses XLSX files and extracts structured data
3. **Generator Layer**: Converts parsed data to SQL INSERT statements
4. **Output Layer**: Writes SQL to file or stdout

### Module Structure

```
src/
├── main.rs              # Entry point and CLI handling
├── input/
│   ├── mod.rs          # Input module interface
│   └── file_handler.rs # File input handling
├── parser/
│   ├── mod.rs          # Parser module interface
│   ├── xlsx_parser.rs  # XLSX parsing logic
│   └── data_model.rs   # Data structures
├── generator/
│   ├── mod.rs          # Generator module interface
│   ├── sql_generator.rs # SQL generation logic
│   └── formatter.rs    # SQL formatting utilities
└── output/
    ├── mod.rs          # Output module interface
    └── writer.rs       # File/stdout writing
```

## Data Model

### Core Data Structures

```rust
use calamine::{DataType, Range};

#[derive(Debug, Clone)]
pub struct WorkbookData {
    pub sheets: Vec<SheetData>,
}

#[derive(Debug, Clone)]
pub struct SheetData {
    pub name: String,
    pub range: Range<DataType>,
}

#[derive(Debug)]
pub struct SqlStatement {
    pub table_name: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<SqlValue>>,
}

#[derive(Debug, Clone)]
pub enum SqlValue {
    Text(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    DateTime(String),
    Null,
}

impl From<&DataType> for SqlValue {
    fn from(data_type: &DataType) -> Self {
        match data_type {
            DataType::Empty => SqlValue::Null,
            DataType::String(s) => SqlValue::Text(s.clone()),
            DataType::Float(f) => SqlValue::Number(*f),
            DataType::Int(i) => SqlValue::Integer(*i),
            DataType::Bool(b) => SqlValue::Boolean(*b),
            DataType::DateTime(dt) => SqlValue::DateTime(dt.to_string()),
            DataType::Error(_) => SqlValue::Null,
            DataType::DateTimeIso(dt) => SqlValue::DateTime(dt.clone()),
            DataType::DurationIso(dur) => SqlValue::Text(dur.clone()),
        }
    }
}
```

## Component Design

### 1. Input Layer

**Responsibilities:**
- Handle command-line arguments
- Support file input via `-f` option
- Handle drag-and-drop file operations
- Read from stdin when appropriate

**Interface:**
```rust
pub trait InputHandler {
    fn get_file_path(&self) -> Result<PathBuf, InputError>;
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, InputError>;
}

pub struct FileInputHandler;
```

### 2. Parser Layer

**Responsibilities:**
- Parse XLSX file format using calamine
- Extract sheet names and data ranges
- Handle calamine's DataType variants
- Provide error handling for malformed files

**Interface:**
```rust
use calamine::{open_workbook_auto, Reader, Error as CalamineError};

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
            if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
                sheets.push(SheetData {
                    name: sheet_name,
                    range,
                });
            }
        }
        
        Ok(WorkbookData { sheets })
    }
}
```

**Key Features:**
- Uses `open_workbook_auto` for automatic file type detection
- Leverages calamine's `Range<DataType>` for efficient data access
- Supports all calamine-compatible formats (xlsx, xls, xlsm, xlsb, ods)
- Handles calamine's comprehensive DataType enum
- Efficient iteration over rows using calamine's row iterator

**Data Processing:**
```rust
impl SheetData {
    pub fn get_columns(&self) -> Result<Vec<String>, ParseError> {
        if let Some(first_row) = self.range.rows().next() {
            Ok(first_row.iter()
                .map(|cell| match cell {
                    DataType::String(s) => s.clone(),
                    DataType::Empty => String::new(),
                    other => format!("{}", other),
                })
                .collect())
        } else {
            Err(ParseError::EmptySheet)
        }
    }
    
    pub fn get_data_rows(&self) -> impl Iterator<Item = &[DataType]> {
        self.range.rows().skip(1) // Skip header row
    }
}

### 3. Generator Layer

**Responsibilities:**
- Convert calamine DataType values to SQL INSERT statements
- Handle SQL value escaping and formatting
- Generate table-specific INSERT statements
- Support MySQL-style backtick quoting

**Interface:**
```rust
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
                let row_values: Vec<SqlValue> = row.iter()
                    .map(SqlValue::from)
                    .collect();
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
        
        Ok(statements)
    }
    
    fn format_statement(&self, statement: &SqlStatement) -> String {
        let table_name = format!("`{}`", statement.table_name);
        let columns = statement.columns.iter()
            .map(|col| format!("`{}`", col))
            .collect::<Vec<_>>()
            .join(", ");
        
        let values_str = statement.values.iter()
            .map(|row| {
                let row_str = row.iter()
                    .map(|val| self.format_sql_value(val))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("({})", row_str)
            })
            .collect::<Vec<_>>()
            .join(",\n");
        
        format!("INSERT INTO {} ({}) VALUES\n{};", table_name, columns, values_str)
    }
}

impl MySqlGenerator {
    fn format_sql_value(&self, value: &SqlValue) -> String {
        match value {
            SqlValue::Null => "NULL".to_string(),
            SqlValue::Text(s) => format!("'{}'", s.replace("'", "''")),
            SqlValue::Number(f) => f.to_string(),
            SqlValue::Integer(i) => i.to_string(),
            SqlValue::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
            SqlValue::DateTime(dt) => format!("'{}'", dt),
        }
    }
}
```

**SQL Generation Rules:**
- Table names are wrapped in backticks: `` `table_name` ``
- Column names are wrapped in backticks: `` `column_name` ``
- String values are single-quoted and escaped
- NULL values for empty cells (DataType::Empty)
- Proper handling of Unicode characters
- Boolean values converted to 1/0 for MySQL compatibility
- DateTime values formatted as strings

### 4. Output Layer

**Responsibilities:**
- Write generated SQL to file or stdout
- Handle file creation and error reporting
- Support different output formats if needed

**Interface:**
```rust
pub trait OutputWriter {
    fn write(&self, content: &str, destination: &OutputDestination) -> Result<(), OutputError>;
}

pub enum OutputDestination {
    File(PathBuf),
    Stdout,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum Xlsx2SqlError {
    #[error("Input error: {0}")]
    Input(#[from] InputError),
    
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    
    #[error("Calamine error: {0}")]
    Calamine(#[from] calamine::Error),
    
    #[error("Generation error: {0}")]
    Generator(#[from] GeneratorError),
    
    #[error("Output error: {0}")]
    Output(#[from] OutputError),
}

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid file format")]
    InvalidFormat,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid XLSX format")]
    InvalidFormat,
    
    #[error("Sheet has no data")]
    EmptySheet,
    
    #[error("Missing column headers")]
    MissingHeaders,
    
    #[error("Calamine error: {0}")]
    CalamineError(#[from] calamine::Error),
}
```

## CLI Interface

### Command Line Options

```
xlsx2sql [OPTIONS] [FILE]

Arguments:
  [FILE]  Input XLSX file path

Options:
  -f, --file <FILE>     Input XLSX file path
  -o, --output <FILE>   Output SQL file path (default: stdout)
  -h, --help            Print help information
  -V, --version         Print version information
```

### Usage Examples

```bash
# From file argument
xlsx2sql data.xlsx

# Using -f option
xlsx2sql -f data.xlsx

# With output file
xlsx2sql -f data.xlsx -o output.sql

# From stdin (drag and drop)
cat data.xlsx | xlsx2sql
```

## Dependencies

### Required Crates

```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
calamine = "0.25"
thiserror = "1.0"
anyhow = "1.0"
```

**Dependency Justification:**
- `clap`: Modern CLI argument parsing with derive macros
- `calamine`: Pure Rust XLSX/XLS parser with excellent performance and comprehensive format support
  - Supports xlsx, xls, xlsm, xlsb, xla, xlam, ods formats
  - Provides efficient `Range<DataType>` for data access
  - Handles all Excel data types including dates, errors, and formulas
  - Lazy loading support for large files
  - Pure Rust implementation with no external dependencies
- `thiserror`: Ergonomic error handling
- `anyhow`: Error context and chaining

**Calamine Performance Benefits:**
Based on calamine's benchmarks, it significantly outperforms other Excel libraries:
- 1.75x faster than Go's excelize
- 7.05x faster than C#'s ClosedXML  
- 9.43x faster than Python's openpyxl
- Processes ~1.1M cells per second
- Low memory usage with efficient Vec growth patterns

## Implementation Phases

### Phase 1: Core Functionality
- Basic XLSX parsing using calamine's `open_workbook_auto`
- Simple SQL generation for single sheet using calamine's DataType
- File input/output with path-based access
- Basic error handling with calamine::Error integration

**Implementation Example:**
```rust
use calamine::{open_workbook_auto, Reader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = open_workbook_auto("input.xlsx")?;
    
    for sheet_name in workbook.sheet_names().to_owned() {
        if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
            // Extract headers from first row
            let headers: Vec<String> = range.rows().next()
                .unwrap_or(&[])
                .iter()
                .map(|cell| cell.to_string())
                .collect();
            
            // Generate SQL for each data row
            for row in range.rows().skip(1) {
                // Convert row to SQL INSERT statement
            }
        }
    }
    
    Ok(())
}
```

### Phase 2: Enhanced Features
- Multiple sheet support with calamine's sheet iteration
- Improved DataType handling for all calamine variants
- Better SQL formatting with proper escaping
- Command-line options and file type auto-detection
- Support for all calamine-supported formats (xls, xlsx, xlsm, xlsb, ods)

### Phase 3: Robustness
- Comprehensive error handling
- Input validation
- Performance optimization
- Testing coverage

### Phase 4: Polish
- Documentation
- Examples
- Performance benchmarks
- CI/CD setup

## Testing Strategy

### Unit Tests
- Parser component with various XLSX/XLS/ODS formats supported by calamine
- SQL generator with different calamine DataType variants
- Error handling scenarios including calamine::Error cases
- Unicode character handling (Japanese text from spec)
- DataType conversion testing (Empty, String, Float, Int, Bool, DateTime, etc.)

**Example Test Cases:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use calamine::DataType;

    #[test]
    fn test_datatype_conversion() {
        assert_eq!(SqlValue::from(&DataType::Empty), SqlValue::Null);
        assert_eq!(SqlValue::from(&DataType::String("test".to_string())), 
                   SqlValue::Text("test".to_string()));
        assert_eq!(SqlValue::from(&DataType::Float(3.14)), SqlValue::Number(3.14));
        assert_eq!(SqlValue::from(&DataType::Int(42)), SqlValue::Integer(42));
        assert_eq!(SqlValue::from(&DataType::Bool(true)), SqlValue::Boolean(true));
    }

    #[test]
    fn test_japanese_characters() {
        let japanese_text = "業務用";
        let data_type = DataType::String(japanese_text.to_string());
        let sql_value = SqlValue::from(&data_type);
        match sql_value {
            SqlValue::Text(s) => assert_eq!(s, japanese_text),
            _ => panic!("Expected Text variant"),
        }
    }
}

### Integration Tests
- End-to-end file processing
- CLI interface testing
- Output validation

### Test Data
- Sample XLSX files with various data types
- Unicode character test cases
- Edge cases (empty sheets, missing headers)
- Large file performance tests

## Performance Considerations

### Memory Usage
- Leverage calamine's efficient Range<DataType> structure
- Use calamine's lazy loading capabilities for large files
- Benefit from calamine's optimized memory usage patterns
- Stream processing approach with row iteration rather than loading entire sheets

**Calamine Memory Advantages:**
- Calamine shows efficient memory usage with stepping patterns (Vec growth and freeing)
- Memory usage drops after processing compared to garbage-collected alternatives
- No temporary file creation for large files (unlike some competitors)

### Processing Speed
- Benefit from calamine's superior performance (1.1M+ cells/second)
- Parallel sheet processing using calamine's sheet iteration
- Optimized SQL string concatenation
- Minimal allocations in hot paths using calamine's efficient data structures

### File Size Limits
- Support for large files leveraging calamine's efficient processing
- Based on calamine benchmarks: tested with 186MB+ files (1M+ rows, 41 columns)
- Streaming approach using calamine's row iterator for memory efficiency
- No artificial limits imposed by the parser layer

**Calamine Performance Reference:**
- Successfully handles 1,000,001 rows × 41 columns (41M cells)
- Processes 28M+ valued cells efficiently
- Linear memory usage without garbage collection overhead

## Security Considerations

### Input Validation
- Validate file format using calamine's auto-detection capabilities
- Leverage calamine's comprehensive format support (xlsx, xls, xlsm, xlsb, ods)
- Sanitize sheet and column names for SQL injection prevention
- Limit file size to prevent DoS attacks (based on calamine's proven performance)

### SQL Generation
- Proper escaping of string values
- Validate identifiers (table and column names)
- No dynamic SQL construction from user input

## Future Enhancements

### Potential Features
- Support for other spreadsheet formats already supported by calamine (CSV via external crate, ODS native)
- Configuration file for custom mappings
- SQL dialect support (PostgreSQL, SQLite, etc.) with different generators
- Data type hints and constraints based on calamine DataType analysis
- Batch processing of multiple files using calamine's efficient processing
- GUI interface
- Web service API
- Integration with calamine's advanced features (VBA extraction, defined names, formulas)

### Performance Improvements
- Leverage calamine's already superior performance (7x+ faster than alternatives)
- Parallel processing of multiple sheets using calamine's efficient sheet access
- Memory-mapped file access if needed (calamine handles this internally)
- Incremental processing for very large files using calamine's row iteration
- Output compression for large SQL files

**Calamine Baseline Performance:**
- Current performance: ~1.1M cells/second processing rate
- Memory efficient with optimized allocation patterns
- No disk I/O for temporary storage during processing
- Superior to major alternatives in all benchmarks

## Conclusion

This design provides a solid foundation for the xlsx2sql tool, focusing on:
- Clean separation of concerns with calamine integration
- Robust error handling including calamine-specific errors
- Extensible architecture leveraging calamine's capabilities
- Performance advantages from calamine's proven efficiency
- Security best practices with proper data validation

The modular design allows for incremental development and future enhancements while maintaining code quality and reliability. By leveraging calamine, we benefit from:

**Key Advantages:**
- **Performance**: 7-9x faster than major alternatives
- **Compatibility**: Support for xlsx, xls, xlsm, xlsb, ods formats
- **Memory Efficiency**: Optimized allocation patterns and lazy loading
- **Pure Rust**: No external dependencies, excellent integration
- **Comprehensive DataType Support**: Handles all Excel data types including dates, errors, and Unicode
- **Battle-tested**: Proven performance with large files (1M+ rows, 186MB+ files)

This positions xlsx2sql as a high-performance, reliable tool for Excel-to-SQL conversion with excellent scalability and format support.
