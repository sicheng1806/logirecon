//! 数据解析器
//!
//! # Example
//! ```
//! use logirecon::parser::{WBParser, WBParseConfig, Parse};
//! use logirecon::validate::IntoValidated;
//! use polars::prelude::*;
//! fn main() {
//!     let data: DataFrame = df!(
//!         "发货日期" => ["2026/04/04",],
//!         "运单号" => ["WB2604024559"],
//!         "订单号" => ["FBA199JBH82C,FBA199JCMDW7,FBA199KNT8RY"],
//!         "仓库编码" => ["GEU2"],
//!         "件数" => [141],
//!         "收费重" => [1899],
//!         "单价" => ["3.60/KG"],
//!     )
//!     .unwrap();
//!     let mut config = WBParseConfig::default();
//!     config.headers.date = "发货日期".to_string();
//!     let bill = WBParser::parse(data, config).unwrap();
//!     println!("{}", &bill.0);
//!     let data = bill.into_validated().unwrap();
//!     println!("{}", data);
//! }
//! ```

mod ddd;
mod headers;
mod headway;
mod parse;
mod ts;
mod wb;

pub use ddd::{DDDHeaders, DDDParseConfig, DDDParser};
pub use headers::AsHeaders;
pub use headway::{HeadwayHeaders, HeadwayParseConfig, HeadwayParser};
pub use parse::Parse;
pub use ts::{TSHeaders, TSParseConfig, TSParser};
pub use wb::{WBHeaders, WBParseConfig, WBParser};
