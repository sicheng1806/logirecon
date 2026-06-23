//！ 用于从各种数据载体读取所需数据的IO功能

pub mod excel;
mod excel_impl;
pub use excel::ExcelReader;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExcelError {
    #[error("重复载入数据，可能会覆盖之前载入的数据")]
    DuplicateLoad,
    #[error("载入数据时发生错误,{0}")]
    Load(#[from] calamine::Error),
    #[error("还未载入数据")]
    NotLoad,
    #[error("查找数据时发生错误")]
    Find(String),
    #[error("转换数据时发生错误")]
    Transform(#[from] polars::error::PolarsError),
}
