use std::collections::HashMap;

use super::{ReconsileColumn, Reconsiler};
use crate::{DataFrame, Error, Result};

/// 生成 [Reconsiler] 的帮助类
///
/// # Example
///
/// ```
/// use std::error::Error;
/// use logirecon::reconsile::{ReconsileOption, ReconsileColumn};
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     use polars::prelude::*;
///
///     let df1: DataFrame = df!("Score" => [99.0, 81.5, 75.],
///                              "No" => ["1", "2", "3"],
///                              "Name" => ["李勇", "张三", "李四"])?;
///
///     let df2: DataFrame = df!("Score" => [99.5, 81.5, 75.],
///                               "No" => ["2", "1", "3"],
///                              "Name" => ["李勇", "张三", "胡五"])?;
///
///     let reconsiler = ReconsileOption::new_with_columns([
///         ("Score", ReconsileColumn::Numberic(0.1)),
///         ("Name", ReconsileColumn::PK),
///         ("No", ReconsileColumn::Exact),
///     ])
///     .left(df1, "A")
///     .right(df2, "B")
///     .try_into_reconsiler()?
///     .build_result()?;
///
///     let width_res = reconsiler.get_width_result()?;
///     println!("width result : {}", width_res);
///     let long_res = reconsiler.get_long_result()?;
///     println!("long result : {}", long_res);
///     Ok(())
/// }
/// ```
pub struct ReconsileOption {
    columns: HashMap<String, ReconsileColumn>,
    left: Option<(String, DataFrame)>,
    right: Option<(String, DataFrame)>,
}

impl ReconsileOption {
    pub fn new_with_columns<S, I>(iter: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = (S, ReconsileColumn)>,
    {
        Self {
            columns: iter.into_iter().map(|(n, c)| (n.into(), c)).collect(),
            left: None,
            right: None,
        }
    }

    pub fn left(mut self, df: DataFrame, name: impl Into<String>) -> Self {
        self.left = Some((name.into(), df));
        self
    }

    pub fn right(mut self, df: DataFrame, name: impl Into<String>) -> Self {
        self.right = Some((name.into(), df));
        self
    }

    pub fn try_into_reconsiler(self) -> Result<Reconsiler> {
        let columns = self.columns;
        let left = self
            .left
            .ok_or(Error::Impl("还未设置左部DataFrame".into()))?;
        let right = self
            .right
            .ok_or(Error::Impl("还未设置右部DataFrame".into()))?;
        Ok(Reconsiler::new(columns, left, right)?)
    }
}
