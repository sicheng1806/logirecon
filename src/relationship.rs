//! 关系数据表
//!
//! 储存报关周次、运单号、货件单号的关系

use crate::{Error, RELATIONSHIP_SCHEMA, Result, DataFrame, LazyFrame, Standardlize};

#[derive(Default)]
pub struct RelationShip {
    dataframe: Option<LazyFrame>,
}

impl RelationShip {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加运单号和货运单号，从表格中读取货运单号和运单号
    pub fn add_waybill_no<S: Into<String>>(
        &mut self,
        df: &DataFrame,
        shipment_no: S,
        waybill_no: S,
    ) -> Result<&mut Self> {
        use polars::prelude::*;

        let df = df.clone().lazy().select([
            col(shipment_no.into()).alias("货运单号"),
            col(waybill_no.into()).alias("运单号"),
        ]);
        self.dataframe = if self.dataframe.is_none() {
            Some(df)
        } else {
            Some(
                self.dataframe
                    .clone()
                    .unwrap()
                    .full_join(df, "货运单号", "货运单号"),
            )
        };
        Ok(self)
    }

    /// 添加报关周次, 根据已有货运单号添加报关周次
    pub fn add_customs_no<S: Into<String>>(
        &mut self,
        df: &DataFrame,
        shipment_no: S,
        customs_no: S,
    ) -> Result<&mut Self> {
        use polars::prelude::*;
        if self.dataframe.is_none() {
            return Err(Error::Process("实现错误：当前关系表中还未有数据".into()));
        }
        let df = df.clone().lazy().select([
            col(shipment_no.into()).alias("货运单号"),
            col(customs_no.into()).alias("报关周次"),
        ]);
        self.dataframe = Some(self.dataframe.clone().unwrap().left_join(
            df,
            "货运单号",
            "货运单号",
        ));
        Ok(self)
    }

    /// 返回健全的关系，即运单号、货运单号、报关周次都具有的行
    pub fn get_relation(&self) -> Result<DataFrame> {
        use polars::prelude::*;
        let df = self.dataframe.clone().unwrap();
        let df = RELATIONSHIP_SCHEMA
            .standardlize(df)?
            .filter(col("货运单号").is_not_null())
            .filter(col("报关周次").is_not_null())
            .filter(col("运单号").is_not_null())
            .collect()?;
        Ok(df)
    }
}