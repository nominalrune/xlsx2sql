use thiserror::Error;

#[derive(Debug, Error)]
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

#[derive(Debug, Error)]
pub enum InputError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid file format")]
    InvalidFormat,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
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

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("No data to generate SQL from")]
    NoData,
    
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
}

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("Failed to write output: {0}")]
    WriteError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
