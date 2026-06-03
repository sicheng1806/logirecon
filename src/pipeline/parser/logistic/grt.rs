use std::collections::HashMap;

use polars::frame::DataFrame;

use crate::{Result, excel::ExcelReadOptions};

pub struct GRTParser {
    headers: HashMap<String, String>,
    opts: ExcelReadOptions,
    datefmt: String,
    units: (String, String),
}

impl Default for GRTParser {
    fn default() -> Self {
        let headers = [
            "日期",
            "运单号",
            "扩展单号",
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
            opts: ExcelReadOptions::default()
                .with_headers(headers)
                .with_primary("序号"),
            datefmt: "%Y/%m/%d".into(),
            units: ("KG".into(), "票".into()),
        }
    }
}

impl GRTParser {
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

    /// 设置文件路径
    pub fn with_path(&mut self, path: impl Into<String>) -> &mut Self {
        todo!()
    }

    // 设置表名称
    pub fn with_sheet(&mut self, name: impl Into<String>) -> &mut Self {
        todo!()
    }

    // 设置日期格式
    pub fn with_datefmt(&mut self, fmt: impl Into<String>) -> &mut Self {
        todo!()
    }

    // 设置价格解析单位
    pub fn with_unit(&mut self, freight: impl Into<String>, customs: impl Into<String>) -> &mut Self {
        todo!()
    }

    // 读取数据表
    pub fn datafram(&self) -> Result<DataFrame> {
        todo!()
    }
}