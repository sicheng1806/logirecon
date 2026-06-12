use std::collections::HashMap;

use super::{AggOption, BILL_SCHEMA, SHIPMENT_SCHEMA};
use crate::DataType;
use crate::{DataFrame, IntoLazy, LazyFrame, Result};

/// 帮助类型: HashMap<String, (DataType, AggOption)>
pub struct DataSchema(HashMap<String, (DataType, AggOption)>);

impl DataSchema {
    pub fn dtypes(&self) -> impl Iterator<Item = (&String, &DataType)> {
        self.0.iter().map(|(name, (dtype, _))| (name, dtype))
    }

    pub fn agg_options(&self) -> impl Iterator<Item = (&String, &AggOption)> {
        self.0.iter().map(|(name, (_, agg))| (name, agg))
    }

    pub fn validate(&self, df: LazyFrame) -> Result<DataFrame> {
        use polars::prelude::*;
        let cast_expr: Vec<_> = self
            .dtypes()
            .map(|(name, dtype)| col(name).cast(dtype.to_owned()).alias(name))
            .collect();
        let groupby_expr: Vec<_> = self
            .agg_options()
            .filter(|(_, agg)| **agg == AggOption::PK)
            .map(|(name, _)| col(name))
            .collect();
        let agg_expr: Vec<_> = self
            .agg_options()
            .filter(|(_, agg)| **agg != AggOption::PK)
            .map(|(name, agg)| match agg {
                AggOption::BySum => col(name).sum(),
                _ => col(name).first_non_null(),
            })
            .collect();
        let res = df
            .select(cast_expr)
            .group_by(groupby_expr)
            .agg(agg_expr)
            .collect()?;
        Ok(res)
    }
}

impl FromIterator<(String, (DataType, AggOption))> for DataSchema {
    fn from_iter<T: IntoIterator<Item = (String, (DataType, AggOption))>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

/// 表示验证后的数据
///
/// 具有明确的主键和列名和对应的类型
///
/// # Example
///
/// ```ignore
/// use logirecon::BillValicated;
///
/// let data = BillValicated::with_dataframe(df);
/// let valicated = data.get_valicated().unwrap();
/// println!("{}", valicated);
/// ```
/// 载入数据
pub trait Validated {
    fn with_dataframe(df: DataFrame) -> Self;

    /// 返回验证的数据
    fn get_valicated(&self) -> Result<DataFrame>;
}

pub struct BillValidated {
    dataframe: LazyFrame,
}

pub struct ShipmentValidated {
    dataframe: LazyFrame,
}

impl Validated for BillValidated {
    fn with_dataframe(df: DataFrame) -> Self {
        Self {
            dataframe: df.lazy(),
        }
    }

    fn get_valicated(&self) -> Result<DataFrame> {
        BILL_SCHEMA.validate(self.dataframe.clone())
    }
}

impl Validated for ShipmentValidated {
    fn with_dataframe(df: DataFrame) -> Self {
        Self {
            dataframe: df.lazy(),
        }
    }

    fn get_valicated(&self) -> Result<DataFrame> {
        SHIPMENT_SCHEMA.validate(self.dataframe.clone())
    }
}
