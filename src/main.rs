use clap::Parser;
use std::path::PathBuf;

mod errors;
mod parser;
mod generator;
mod input;
mod output;

use errors::Xlsx2SqlError;
use parser::{CalamineXlsxParser, XlsxParser};
use generator::{MySqlGenerator, SqlGenerator};
use output::{FileOutputWriter, OutputWriter, OutputDestination};

#[derive(Parser)]
#[command(name = "xlsx2sql")]
#[command(about = "Convert xlsx files to SQL INSERT statements")]
#[command(version = "0.1.0")]
struct Cli {
    /// Input XLSX file path
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Input XLSX file path (alternative to positional argument)
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    file_option: Option<PathBuf>,

    /// Output SQL file path (default: input filename with .sql extension)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Xlsx2SqlError> {
    let cli = Cli::parse();

    // Determine input file path
    let input_path = cli.file.or(cli.file_option).ok_or_else(|| {
        Xlsx2SqlError::Input(errors::InputError::FileNotFound(
            "No input file specified. Use: xlsx2sql <file> or xlsx2sql -f <file>".to_string()
        ))
    })?;

    // Parse the XLSX file
    let parser = CalamineXlsxParser;
    let workbook_data = parser.parse(&input_path)?;

    // Generate SQL statements
    let generator = MySqlGenerator;
    let statements = generator.generate(&workbook_data)?;

    // Format SQL output
    let mut output_content = String::new();
    for statement in statements {
        output_content.push_str(&generator.format_statement(&statement));
        output_content.push_str("\n\n");
    }

    // Write output
    let writer = FileOutputWriter;
    let destination = match cli.output {
        Some(path) => OutputDestination::File(path),
        None => {
            // Generate output filename by replacing .xlsx with .sql
            let mut output_path = input_path.clone();
            output_path.set_extension("sql");
            OutputDestination::File(output_path)
        },
    };

    writer.write(&output_content, &destination)?;

    Ok(())
}
