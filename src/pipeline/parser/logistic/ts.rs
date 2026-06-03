use std::{collections::HashMap, path::PathBuf};

use polars::frame::DataFrame;

use super::BillType;
use crate::{Result, excel::ExcelReadOptions};

pub struct TSParser {
    headers: HashMap<String, String>,
    primary: String,
    sheets: Vec<(PathBuf, String)>,
    datefmt: String,
}

impl Default for TSParser {
    fn default() -> Self {
        let headers = [
            "日期",
            "运单号",
            "客户运单号",
            "地址编码",
            "件数",
            "收费重",
            "单价",
        ];
        Self {
            headers: HashMap::from_iter(
                headers
                    .clone()
                    .into_iter()
                    .map(|t| (t.to_string(), t.to_string())),
            ),
            datefmt: "%Y-%m-%d".into(),
            primary: "序号".into(),
            sheets: vec![],
        }
    }
}

impl TSParser {
    /// 返回默认表头
    pub fn headers(&self) -> Vec<&String> {
        self.headers.keys().collect()
    }

    /// 返回当前表头
    pub fn current_headers(&self) -> Vec<&String> {
        self.headers.values().collect()
    }

    /// 设置表头
    pub fn with_headers<S: Into<String>>(
        &mut self,
        headers: impl IntoIterator<Item = (S, S)>,
    ) -> &mut Self {
        // 1. 设置表头到 headers 属性
        todo!()
    }

    /// 添加需要解析的表格
    pub fn add_sheet(
        &mut self,
        path: impl Into<PathBuf>,
        sheet: impl Into<String>,
        btype: BillType,
    ) -> &mut Self {
        todo!()
    }

    /// 清空需要解析的表格
    pub fn clear_sheet(&mut self) -> &mut Self {
        todo!()
    }

    // 设置日期格式
    pub fn with_datefmt(&mut self, fmt: impl Into<String>) -> &mut Self {
        todo!()
    }

    // 读取数据表
    pub fn dataframe(&self) -> Result<DataFrame> {
        todo!()
    }
}
