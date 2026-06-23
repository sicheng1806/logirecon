use std::collections::HashMap;

use super::{ReconcileColumn, ReconcileError, Reconciler, Result};
use crate::DataFrame;

/// 用于生成 [Reconciler] 的配置类
///
/// # Example
///
/// ```
/// use std::error::Error;
/// use logirecon::reconcile::{ReconcileOption, ReconcileColumn};
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
///     let reconciler = ReconcileOption::new_with_columns([
///         ("Score", ReconcileColumn::Numeric(0.1)),
///         ("Name", ReconcileColumn::PK),
///         ("No", ReconcileColumn::Exact),
///     ])
///     .left(df1, "A")
///     .right(df2, "B")
///     .try_into_reconciler()?
///     .build_result()?;
///
///     let width_res = reconciler.get_width_result()?;
///     println!("width result : {}", width_res);
///     let long_res = reconciler.get_long_result()?;
///     println!("long result : {}", long_res);
///     Ok(())
/// }
/// ```
pub struct ReconcileOption {
    columns: HashMap<String, ReconcileColumn>,
    left: Option<(String, DataFrame)>,
    right: Option<(String, DataFrame)>,
}

impl ReconcileOption {
    pub fn new_with_columns<S, I>(iter: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = (S, ReconcileColumn)>,
    {
        Self {
            columns: iter.into_iter().map(|(n, c)| (n.into(), c)).collect(),
            left: None,
            right: None,
        }
    }

    pub fn freight() -> Self {
        Self::new_with_columns(FREIGHT_RECONCILE_COLUMNS)
    }

    pub fn customs() -> Self {
        Self::new_with_columns(CUSTOMS_RECONCILE_COLUMNS)
    }

    pub fn left(mut self, df: DataFrame, name: impl Into<String>) -> Self {
        self.left = Some((name.into(), df));
        self
    }

    pub fn right(mut self, df: DataFrame, name: impl Into<String>) -> Self {
        self.right = Some((name.into(), df));
        self
    }

    pub fn try_into_reconciler(self) -> Result<Reconciler> {
        let columns = self.columns;
        let left = self
            .left
            .ok_or(ReconcileError::NotSet("还未设置左部DataFrame".into()))?;
        let right = self
            .right
            .ok_or(ReconcileError::NotSet("还未设置右部DataFrame".into()))?;
        Reconciler::new(columns, left, right)
    }
}

pub const FREIGHT_RECONCILE_COLUMNS: [(&str, ReconcileColumn); 8] = [
    ("运单号", ReconcileColumn::PK),
    ("货件单号", ReconcileColumn::None),
    ("日期", ReconcileColumn::None),
    ("物流中心编码", ReconcileColumn::None),
    ("货代名称", ReconcileColumn::None),
    ("件数", ReconcileColumn::Numeric(0.001)),
    ("单价", ReconcileColumn::Numeric(0.001)),
    ("计费重", ReconcileColumn::Numeric(0.001)),
];

pub const CUSTOMS_RECONCILE_COLUMNS: [(&str, ReconcileColumn); 4] = [
    ("报关周次", ReconcileColumn::PK),
    ("运单号", ReconcileColumn::None),
    ("货代名称", ReconcileColumn::None),
    ("金额", ReconcileColumn::Numeric(0.001)),
];
