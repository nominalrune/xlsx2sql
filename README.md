# xlsx2sql

Convert Excel files (.xlsx/.xls) to SQL INSERT statements with support for Japanese characters and various data types.

## Features

- ✅ Convert Excel files to MySQL-compatible SQL INSERT statements
- ✅ Support for Japanese characters and Unicode text
- ✅ Handle various data types (text, numbers, dates, booleans)
- ✅ Automatic table naming based on sheet names
- ✅ Column header detection
- ✅ Robust error handling
- ✅ Cross-platform support (Windows, macOS, Linux)

## Installation

### From Releases (Recommended)

Download the pre-built binary for your platform from the [Releases](https://github.com/nominalrune/xlsx2sql/releases) page:

- **Windows 64-bit**: `xlsx2sql-windows-x64.zip`
- **macOS Intel**: `xlsx2sql-macos-intel.tar.gz`
- **macOS Apple Silicon**: `xlsx2sql-macos-arm64.tar.gz`
- **Linux 64-bit**: `xlsx2sql-linux-x64.tar.gz`

Extract the archive and add the binary to your PATH.

### From Source

```bash
git clone https://github.com/nominalrune/xlsx2sql.git
cd xlsx2sql
cargo build --release
```

The binary will be available at `target/release/xlsx2sql`.

## Usage

### Basic Usage

```bash
# Convert input.xlsx to input.sql
xlsx2sql input.xlsx

# Specify custom output file
xlsx2sql input.xlsx -o custom_output.sql

# Alternative syntax
xlsx2sql -f input.xlsx -o output.sql
```

### Example

Given an Excel file `employees.xlsx` with content:

| id | name     | age | salary |
|----|----------|-----|--------|
| 1  | 田中太郎  | 30  | 50000  |
| 2  | 佐藤花子  | 25  | 45000  |

Running `xlsx2sql employees.xlsx` generates `employees.sql`:

```sql
INSERT INTO `employees` (`id`, `name`, `age`, `salary`) VALUES
(1,'田中太郎',30,50000),
(2,'佐藤花子',25,45000);
```

## Command Line Options

```
xlsx2sql [OPTIONS] [FILE]

Arguments:
  [FILE]  Input XLSX file path

Options:
  -f, --file <FILE>    Input XLSX file path (alternative to positional argument)
  -o, --output <FILE>  Output SQL file path (default: input filename with .sql extension)
  -h, --help           Print help
  -V, --version        Print version
```

## Supported Data Types

- **Text**: Properly escaped with single quotes
- **Numbers**: Integer and floating-point values
- **Dates**: Converted to string format
- **Booleans**: Converted to 1 (true) or 0 (false)
- **Empty cells**: Converted to NULL

## Error Handling

The tool provides comprehensive error messages for common issues:

- Invalid file format (non-Excel files)
- File not found
- Empty sheets or missing headers
- No data to process
- File writing errors

## Architecture

xlsx2sql follows a modular architecture:

- **Input Layer**: File validation and format checking
- **Parser Layer**: Excel file parsing using the `calamine` crate
- **Generator Layer**: SQL statement generation with proper formatting
- **Output Layer**: File writing with error handling

## Dependencies

- [calamine](https://crates.io/crates/calamine) - Fast Excel file parsing
- [clap](https://crates.io/crates/clap) - Command line argument parsing
- [thiserror](https://crates.io/crates/thiserror) - Error handling
- [anyhow](https://crates.io/crates/anyhow) - Error context

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running

```bash
cargo run -- input.xlsx
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Changelog

### v0.1.0
- Initial release
- Basic Excel to SQL conversion
- Support for multiple data types
- Japanese character support
- Cross-platform builds
