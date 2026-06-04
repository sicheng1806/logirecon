use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("数据解析错误: {0}")]
    Process(String),
    #[error("IO错误: {0}")]
    IO(String),
    #[error("{0}")]
    UnKnown(String),
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::UnKnown(value.to_string())
    }
}

impl From<polars::error::PolarsError> for Error {
    fn from(value: polars::error::PolarsError) -> Self {
        let msg = value.to_string();
        Self::Process(format!("Polars错误: {}", msg))
    }
}

impl From<calamine::Error> for Error {
    fn from(value: calamine::Error) -> Self {
        let msg = value.to_string();
        Self::IO(format!("Excel读取/写入错误: {}", msg))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
