use crate::{DataType, Error, LazyFrame, Result};
use std::{collections::HashMap, sync::LazyLock};

pub trait Standardlize {
    fn standardlize(&self, df: LazyFrame) -> Result<LazyFrame>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggOptions {
    BySum,
    PK,
    ByFirst,
}

/// 用于约束表单的形式
///
/// 使用[AggOptions]确保主键约束，使用[DataType]确保列类型
///
/// # Examples
///
/// ```rust
/// use logirecon::pipeline::{Schema, AggOptions};
/// use logirecon::Result;
/// use polars::datatypes::DataType;
///
/// fn main() -> Result<()> {
///     let schema = Schema::default()
///         .with_columns(["列A", "列B"].map(|t| (t, DataType::String)))
///         .with_column("列C", DataType::Float64, AggOptions::ByFirst)
///         .with_agg("列B", AggOptions::BySum)?
///         .with_primary("列A")?
///         .ok()?;
///
///     println!("{:?}", schema);
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Schema {
    columns: HashMap<String, (DataType, AggOptions)>,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            columns: HashMap::new(),
        }
    }
}

impl Schema {
    pub fn with_column(
        mut self,
        name: impl Into<String>,
        dtype: DataType,
        agg: AggOptions,
    ) -> Self {
        self.columns.insert(name.into(), (dtype, agg));
        self
    }

    pub fn with_columns(
        mut self,
        columns: impl IntoIterator<Item = (impl Into<String>, (DataType, AggOptions))>,
    ) -> Self {
        self.columns.extend(
            columns
                .into_iter()
                .map(|(name, (dtype, agg))| (name.into(), (dtype, agg))),
        );
        self
    }

    pub fn with_primary(mut self, name: impl Into<String>) -> Result<Self> {
        self = self.with_agg(name, AggOptions::PK)?;
        Ok(self)
    }

    pub fn with_agg(mut self, name: impl Into<String>, agg: AggOptions) -> Result<Self> {
        if let Some((_, _agg)) = self.columns.get_mut(&name.into()) {
            *_agg = agg;
        } else {
            return Err(Error::Process("设置主键失败: 没有列名".into()));
        }
        Ok(self)
    }

    pub fn ok(self) -> Result<Self> {
        // 至少存在一个主键
        let query: Vec<_> = self
            .columns
            .iter()
            .filter(|(_, (_, agg))| agg == &AggOptions::PK)
            .collect();
        if query.len() == 0 {
            return Err(Error::Process("缺少主键".into()));
        }
        Ok(self)
    }

    /// 返回列名
    pub fn headers(&self) -> Vec<String> {
        self.columns
            .keys()
            .into_iter()
            .map(|t| t.to_string())
            .collect()
    }
}

impl Standardlize for Schema {
    /// 返回标准化后的数据表
    fn standardlize(&self, df: LazyFrame) -> Result<LazyFrame> {
        use polars::prelude::*;
        let dtypes = PlHashMap::from_iter(
            self.columns
                .iter()
                .map(|(name, (dtype, _))| (name.as_str(), dtype.clone())),
        );
        let primaries: Vec<_> = self
            .columns
            .iter()
            .filter(|(_, (_, agg))| *agg == AggOptions::PK)
            .map(|(name, _)| col(name))
            .collect();
        let aggs: Vec<_> = self
            .columns
            .iter()
            .filter(|(_, (_, agg))| *agg != AggOptions::PK)
            .map(|(name, (_, agg))| match *agg {
                AggOptions::ByFirst => col(name).first_non_null().alias(name),
                AggOptions::BySum => col(name).sum().alias(name),
                _ => col(name).first_non_null().alias(name),
            })
            .collect();
        let df = df.cast(dtypes, true).group_by(primaries).agg(aggs);
        Ok(df)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_new() -> Result<()> {
        let schema = Schema::default()
            .with_columns(["列A", "列B"].map(|t| (t, (DataType::String, AggOptions::ByFirst))))
            .with_column("列C", DataType::Float64, AggOptions::ByFirst)
            .with_agg("列B", AggOptions::BySum)?
            .with_primary("列A")?
            .ok()?;
        println!("{:?}", schema);

        Ok(())
    }
}
