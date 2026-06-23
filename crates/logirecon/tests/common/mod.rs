#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::LazyLock;

pub static PATH_HEADWAY: LazyLock<PathBuf> = LazyLock::new(|| {
    [env!("CARGO_MANIFEST_DIR"), "../../data/物流头程明细.xlsm"]
        .iter()
        .collect()
});
pub static PATH_BILLS: LazyLock<PathBuf> = LazyLock::new(|| {
    [env!("CARGO_MANIFEST_DIR"), "../../data/物流账单.xlsx"]
        .iter()
        .collect()
});

pub const SHEET_HEADWAY_2026: &'static str = "亚马逊头程2026";
pub const SHEET_HEADWAY_2025: &'static str = "亚马逊头程2025";
pub const SHEET_WB: &'static str = "万邦2604";
pub const SHEET_GRT: &'static str = "国润通2604";
pub const SHEET_TSYF: &'static str = "天盛运费2604";
pub const SHEET_TSBG: &'static str = "天盛报关费2604";
pub const SHEET_DDD: &'static str = "嘀嗒嘀4月账单";

pub const HEADERS_HEADWAY_2026: [&str; 8] = [
    "报关周次",
    "货件单号",
    "物流中心编码",
    "箱数",
    "货件计费重",
    "物流单价",
    "报关或其他费",
    "提货时间",
];

pub const HEADERS_HEADWAY_2025: [&str; 8] = [
    "报关周次",
    "货件单号",
    "物流中心编码",
    "箱数",
    "货件计费重",
    "物流单价",
    "报关费",
    "提货时间",
];

pub const HEADERS_WB: [&str; 9] = [
    "日期",
    "运单号",
    "订单号",
    "仓库编码",
    "件数",
    "收费重",
    "单价",
    "费用类型",
    "金额",
];
pub const HEADERS_GRT: [&str; 9] = [
    "日期",
    "运单号",
    "扩展单号",
    "地址编码",
    "件数",
    "收费重",
    "单价",
    "费用类型",
    "金额",
];

pub const HEADERS_TSYF: [&str; 8] = [
    "日期",
    "客户运单号",
    "运单号",
    "地址编码",
    "件数",
    "收费重",
    "单价",
    "金额",
];
pub const HEADERS_TSBG: [&str; 8] = [
    "日期",
    "客户运单号",
    "运单号",
    "地址编码",
    "件数",
    "收费重",
    "单价",
    "金额",
];
pub const HEADERS_DDD: [&str; 7] = [
    "签入日期",
    "运单号",
    "FBA单号",
    "目的仓",
    "件数",
    "收费重",
    "计算公式",
];
