//! 账单数据表
//!
//! 用以核对的账单。

use polars::datatypes::DataType;
use polars::prelude::{DataFrame, FrozenCategories, LazyFrame};

use super::{AggOptions, Schema};
use crate::Result;

pub struct Bill {
    schema: Schema,
    dataframe: Option<LazyFrame>,
}

impl Bill {
    pub fn schema() -> Schema {
        let fcats = FrozenCategories::new(["报关费", "运费"]).unwrap();
        Schema::default()
            .with_columns([
                ("运单号", (DataType::String, AggOptions::PK)),
                (
                    "账单类型",
                    (DataType::from_frozen_categories(fcats), AggOptions::PK),
                    // (DataType::String, AggOptions::PK)
                ),
                //
                ("货件单号", (DataType::String, AggOptions::ByFirst)),
                // ("报关周次", (DataType::String, AggOptions::ByFirst)),
                ("日期", (DataType::Date, AggOptions::ByFirst)),
                ("物流中心编码", (DataType::String, AggOptions::ByFirst)),
                ("件数", (DataType::Float64, AggOptions::ByFirst)),
                ("货代名称", (DataType::String, AggOptions::ByFirst)),
                //
                ("单价", (DataType::Float64, AggOptions::BySum)),
                ("计费重", (DataType::Float64, AggOptions::ByFirst)),
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

    /// 添加合法的数据表
    pub fn add(&mut self, df: DataFrame) -> Result<()> {
        use polars::prelude::*;
        if let Some(df_old) = self.dataframe.clone() {
            let df_new = self.schema.standardlize(df.lazy())?;
            self.dataframe = Some(concat(&[df_old, df_new], UnionArgs::default())?);
        } else {
            self.dataframe = Some(self.schema.standardlize(df.lazy())?);
        }
        Ok(())
    }

    pub fn get_waybill(&self) -> Result<DataFrame> {
        todo!()
    }

    pub fn get_customs(&self) -> Result<DataFrame> {
        todo!()
    }
}
