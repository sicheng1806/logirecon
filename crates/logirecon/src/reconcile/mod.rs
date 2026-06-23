//! 对账功能的实现
//!
//! 提供基于 [polars::frame::DataFrame] 的通用对账方案

mod option;
mod reconciler;

// reuse
pub use option::{CUSTOMS_RECONCILE_COLUMNS, FREIGHT_RECONCILE_COLUMNS, ReconcileOption};
pub use reconciler::Reconciler;

type Result<T, E = ReconcileError> = std::result::Result<T, E>;

use thiserror::Error;

/// 数据列的对账方法
#[derive(Debug, Clone, PartialEq)]
pub enum ReconcileColumn {
    /// 作为主键
    PK,
    /// 数值匹配
    Numeric(f64),
    /// 精确匹配
    Exact,
    /// 作为信息字段
    None,
}

#[derive(Debug, Error)]
pub enum ReconcileError {
    #[error("还未设置要比对的数据: {0}")]
    NotSet(String),
    #[error("目前只支持单主键对比器，当前主键数量: {0}")]
    PK(usize),
    #[error("方案和数据框不匹配，无法比较: {0}")]
    NotMatch(String),
    #[error("处理错误,{0}")]
    Process(#[from] polars::error::PolarsError),
    #[error("实现错误: {0}")]
    NotReady(String),
}
