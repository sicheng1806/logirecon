//! excel 文件读取
//!
//! 通过[ExcelReadOptions]设置读取选项
//! 通过[ExcelReader]执行实际的读取请求
//!
//! # Example
//!
//! ```
//! fn main() -> Result<()> {
//!     let df = ExcelReadOptions::default()
//!         .with_headers(["序号", "运单号", "单价", "计费重", "费用类型", "金额"])
//!         .with_path("data/test/物流账单.xlsx")
//!         .with_sheet("万邦2604")
//!         .with_primary("序号")
//!         .try_into_reader()?
//!         .finish()?
//! }
//!
//! ```

mod options;
mod reader;

pub use options::ExcelReadOptions;
pub use reader::ExcelReader;
