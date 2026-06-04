//! 账单数据表
//!
//! 用以核对的账单。

use crate::relationship::RelationShip;
use crate::{BILL_SCHEMA, DataFrame, Error, LazyFrame, Result, Standardlize};

pub struct Bill {
    dataframe: Option<LazyFrame>,
}

impl Bill {
    pub fn new() -> Self {
        Self { dataframe: None }
    }

    /// 添加合法的数据表
    pub fn add(&mut self, df: DataFrame) -> Result<()> {
        use polars::prelude::*;
        if let Some(df_old) = self.dataframe.clone() {
            let df_new = BILL_SCHEMA.standardlize(df.lazy())?;
            self.dataframe = Some(concat(&[df_old, df_new], UnionArgs::default())?);
        } else {
            self.dataframe = Some(BILL_SCHEMA.standardlize(df.lazy())?);
        }
        Ok(())
    }

    /// 返回报关周次和货件单号的关系
    pub fn get_shipment_nos(&self) -> Result<DataFrame> {
        use polars::prelude::*;
        let df = self.try_get_dataframe()?;
        let df = df
            .select([
                col("运单号").alias("运单号"),
                col("货运单号").str().split(lit(",")).alias("货运单号"),
            ])
            .explode(
                cols(["货运单号"]),
                ExplodeOptions {
                    empty_as_null: true,
                    keep_nulls: true,
                },
            )
            .collect()?;
        Ok(df)
    }

    pub fn try_get_dataframe(&self) -> Result<LazyFrame> {
        if self.dataframe.is_none() {
            return Err(Error::Process("实现错误: 还未添加数据".into()));
        }
        Ok(self.dataframe.clone().unwrap())
    }

    /// 补充关系
    pub fn with_relations<S: Into<String>>(&mut self, relation: RelationShip) -> Result<&mut Self> {
        use polars::prelude::*;
        let relation = relation
            .get_relation()?
            .lazy()
            .select([col("运单号"), col("报关周次")]);
        let df = self.try_get_dataframe()?;
        self.dataframe = Some(df.left_join(relation, "运单号", "运单号"));
        Ok(self)
    }

    /// 将账单转换为运单号，需要已经填充过关系
    pub fn get_waybill(&self) -> Result<DataFrame> {
        todo!()
    }

    /// 将账单转换为报关单，需要已经填充过关系
    pub fn get_customs(&self) -> Result<DataFrame> {
        todo!()
    }
}
