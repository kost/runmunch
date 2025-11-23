use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunmunchError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Invalid affix format: {0}")]
    InvalidAffix(String),
    
    #[error("Invalid dictionary format: {0}")]
    InvalidDictionary(String),
    
    #[error("No affix file loaded")]
    NoAffixFile,
    
    #[error("No dictionary loaded")]
    NoDictionary,
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("Invalid flag: {0}")]
    InvalidFlag(String),
}

pub type Result<T> = std::result::Result<T, RunmunchError>;