//! 货件数据表
//!
//! 用以核对的货件数据
//!
//! 来源于头程明细

use crate::{Error, LazyFrame, Result, SHIPMENT_SCHEMA};

pub struct Shipment {
    dataframe: Option<LazyFrame>,
}

impl Shipment {
    pub fn new() -> Self {
        Self { dataframe: None }
    }
}
