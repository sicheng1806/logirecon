use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not Found Error: {0}")]
    NotFoundError(String),
    #[error("Parse Error: {0}")]
    ParseError(String),
    #[error("Polars Error: {0}")]
    Polars(#[from] polars::error::PolarsError),
    #[error("calamine Error: {0}")]
    Calamine(#[from] calamine::Error),
    #[error("Other Error: {0}")]
    Other(String),
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::Other(value.to_string())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
