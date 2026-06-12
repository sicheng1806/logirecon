mod option;
mod reconsiler;

// reuse
pub use option::ReconsileOption;
pub use reconsiler::Reconsiler;

/// 表示对账器各列的对账方法
#[derive(Debug, Clone, PartialEq)]
pub enum ReconsileColumn {
    /// 主键
    PK,
    /// 数值匹配
    Numberic(f64),
    /// 精确匹配
    Exact,
    /// 作为信息字段
    None,
}

//
pub const FREIGHT_RECONSILE_COLUMNS: [(&str, ReconsileColumn); 8] = [
    ("运单号", ReconsileColumn::PK),
    ("货件单号", ReconsileColumn::None),
    ("日期", ReconsileColumn::None),
    ("物流中心编码", ReconsileColumn::None),
    ("货代名称", ReconsileColumn::None),
    ("件数", ReconsileColumn::Numberic(0.001)),
    ("单价", ReconsileColumn::Numberic(0.001)),
    ("计费重", ReconsileColumn::Numberic(0.001)),
];

pub const CUSTOMS_RECONSILE_COLUMNS: [(&str, ReconsileColumn); 4] = [
    ("报关周次", ReconsileColumn::PK),
    ("运单号", ReconsileColumn::None),
    ("货代名称", ReconsileColumn::None),
    ("金额", ReconsileColumn::Numberic(0.001)),
];
