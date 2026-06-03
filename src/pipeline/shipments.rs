//! 货件数据表
//!
//! 用以核对的货件数据
//!
//! 来源于头程明细

use polars::lazy::frame::LazyFrame;

use super::{AggOptions, DataType, Schema};
use crate::Result;

pub struct Shipment {
    schema: Schema,
    dataframe: Option<LazyFrame>,
}

impl Shipment {
    pub fn schema() -> Schema {
        Schema::default()
            .with_columns([
                ("货件单号", (DataType::String, AggOptions::PK)),
                ("报关周次", (DataType::String, AggOptions::ByFirst)),
                ("日期", (DataType::Date, AggOptions::ByFirst)),
                ("物流中心编码", (DataType::String, AggOptions::ByFirst)),
                ("箱数", (DataType::Float64, AggOptions::BySum)),
                //
                ("计费重", (DataType::Float64, AggOptions::BySum)),
                ("单价", (DataType::Float64, AggOptions::BySum)),
                ("报关费", (DataType::Float64, AggOptions::ByFirst)),
            ])
            .ok()
            .unwrap()
    }
    pub fn new() -> Self {
        Self {
            schema: Self::schema(),
            dataframe: None,
        }
    }
}
