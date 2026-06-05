use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("数据解析错误: {0}")]
    Process(String),
    #[error("文件读写错误: {0}")]
    IO(String),
    #[error("{0}")]
    Other(String),
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::Other(value.to_string())
    }
}

impl From<polars::error::PolarsError> for Error {
    fn from(value: polars::error::PolarsError) -> Self {
        let msg = value.to_string();
        Self::Process(msg)
    }
}

impl From<calamine::Error> for Error {
    fn from(value: calamine::Error) -> Self {
        let msg = value.to_string();
        Self::IO(msg)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
