use clap::Parser;
use dialoguer::Select;
use std::fs;
use std::path::PathBuf;

mod errors;
mod generator;
mod input;
mod output;
mod parser;

use errors::Xlsx2SqlError;
use generator::{MySqlGenerator, SqlGenerator};
use output::{FileOutputWriter, OutputDestination, OutputWriter};
use parser::{CalamineXlsxParser, XlsxParser};

#[derive(Parser)]
#[command(name = "xlsx2sql")]
#[command(about = "Convert xlsx files to SQL INSERT statements")]
#[command(version = "0.1.6")]
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

fn find_xlsx_files() -> Result<Vec<PathBuf>, Xlsx2SqlError> {
    let mut xlsx_files = Vec::new();

    let entries = fs::read_dir(".").map_err(|e| Xlsx2SqlError::Input(errors::InputError::Io(e)))?;

    for entry in entries {
        let entry = entry.map_err(|e| Xlsx2SqlError::Input(errors::InputError::Io(e)))?;
        let path = entry.path();

        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if ext == "xlsx" || ext == "xls" {
                xlsx_files.push(path);
            }
        }
    }

    xlsx_files.sort();
    Ok(xlsx_files)
}

fn select_input_file() -> Result<PathBuf, Xlsx2SqlError> {
    let xlsx_files = find_xlsx_files()?;

    if xlsx_files.is_empty() {
        return Err(Xlsx2SqlError::Input(errors::InputError::FileNotFound(
            "No Excel files (.xlsx/.xls) found in current directory".to_string(),
        )));
    }

    if xlsx_files.len() == 1 {
        println!("Found Excel file: {}", xlsx_files[0].display());
        return Ok(xlsx_files[0].clone());
    }

    // Convert PathBuf to strings for display
    let file_names: Vec<String> = xlsx_files
        .iter()
        .map(|path| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    let selection = Select::new()
        .with_prompt("Select an Excel file to convert:")
        .items(&file_names)
        .interact()
        .map_err(|e| {
            Xlsx2SqlError::Input(errors::InputError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e.to_string(),
            )))
        })?;

    Ok(xlsx_files[selection].clone())
}

fn main() -> Result<(), Xlsx2SqlError> {
    let cli = Cli::parse();

    // Determine input file path
    let input_path = match cli.file.or(cli.file_option) {
        Some(path) => path,
        None => {
            // No input file specified, try interactive selection
            select_input_file()?
        }
    };

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
        }
    };

    writer.write(&output_content, &destination)?;

    Ok(())
}
