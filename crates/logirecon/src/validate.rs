use crate::DataFrame;
pub use polars::datatypes::DataType;
use std::{collections::HashMap, sync::LazyLock};

pub static BILL_SCHEMA: LazyLock<SchemaValidator> = LazyLock::new(|| {
    use polars::prelude::FrozenCategories;
    let fcats = FrozenCategories::new(["运费", "报关费"]).unwrap();
    let iter = [
        ("运单号", (DataType::String, AggOption::PK)),
        (
            "账单类型",
            (DataType::from_frozen_categories(fcats), AggOption::PK),
            // (DataType::String, AggOptions::PK)
        ),
        //
        ("货件单号", (DataType::String, AggOption::ByFirst)),
        // ("报关周次", (DataType::String, AggOptions::ByFirst)),
        ("日期", (DataType::Date, AggOption::ByFirst)),
        ("物流中心编码", (DataType::String, AggOption::ByFirst)),
        ("件数", (DataType::Float64, AggOption::ByFirst)),
        ("货代名称", (DataType::String, AggOption::ByFirst)),
        //
        ("单价", (DataType::Float64, AggOption::BySum)),
        ("计费重", (DataType::Float64, AggOption::ByFirst)),
    ];
    SchemaValidator::from_iter(iter.map(|(n, s)| (n.to_string(), s)))
});

pub static SHIPMENT_SCHEMA: LazyLock<SchemaValidator> = LazyLock::new(|| {
    let iter = [
        ("货件单号", (DataType::String, AggOption::PK)),
        //
        ("报关周次", (DataType::String, AggOption::ByFirst)),
        ("日期", (DataType::Date, AggOption::ByFirst)),
        ("物流中心编码", (DataType::String, AggOption::ByFirst)),
        //
        ("件数", (DataType::Float64, AggOption::BySum)),
        ("计费重", (DataType::Float64, AggOption::BySum)),
        ("单价", (DataType::Float64, AggOption::ByFirst)),
        ("报关费", (DataType::Float64, AggOption::ByFirst)),
    ];
    SchemaValidator::from_iter(iter.map(|(n, s)| (n.to_string(), s)))
});

/// `IntoValidated` Trait 允许依照规则验证 [DataFrame] 数据
///
/// 实现此Trait的结构体一般用于承载 [DataFrame]，给予 [DataFrame] 数据形式约束和类型绑定
pub trait IntoValidated {
    type Error;

    fn into_validated(self) -> Result<DataFrame, Self::Error>;
}

/// 基于方案的 [DataFrame] 验证器
///
/// 另请参阅 [DataType] 和 [AggOption].
pub struct SchemaValidator(HashMap<String, (DataType, AggOption)>);

/// 聚合选项
///
/// 约定字段在 agg 操作时使用的方案. 请参阅 [DataFrame] 的 agg 方法。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggOption {
    PK,
    ByFirst,
    BySum,
}

impl SchemaValidator {
    pub fn dtypes(&self) -> impl Iterator<Item = (&String, &DataType)> {
        self.0.iter().map(|(name, (dtype, _))| (name, dtype))
    }

    pub fn agg_options(&self) -> impl Iterator<Item = (&String, &AggOption)> {
        self.0.iter().map(|(name, (_, agg))| (name, agg))
    }

    /// 验证给定 DataFrame 是否满足需求
    pub fn validate(&self, df: DataFrame) -> Result<DataFrame, polars::error::PolarsError> {
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
            .lazy()
            .select(cast_expr)
            .group_by(groupby_expr)
            .agg(agg_expr)
            .collect()?;
        Ok(res)
    }
}

impl FromIterator<(String, (DataType, AggOption))> for SchemaValidator {
    fn from_iter<T: IntoIterator<Item = (String, (DataType, AggOption))>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

pub struct BillData(pub DataFrame);
pub struct ShipmentData(pub DataFrame);

impl IntoValidated for BillData {
    type Error = polars::error::PolarsError;

    fn into_validated(self) -> Result<DataFrame, Self::Error> {
        BILL_SCHEMA.validate(self.0)
    }
}

impl IntoValidated for ShipmentData {
    type Error = polars::error::PolarsError;

    fn into_validated(self) -> Result<DataFrame, Self::Error> {
        SHIPMENT_SCHEMA.validate(self.0)
    }
}
