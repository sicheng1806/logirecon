pub mod error;
pub mod excel;
pub mod parser;

// reuse
pub use error::{Error, Result};
pub use excel::{ExcelReadOptions, ExcelReader};
// reuse from other
pub use polars::prelude::{DataFrame, DataType, IntoLazy, LazyFrame, Schema};
