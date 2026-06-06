use std::{collections::HashMap, path::PathBuf};

use crate::{DataFrame, ExcelReadOptions, Result};

/// 指定表头的Excel数据的提取器
///
/// # Example
/// ```no_run
///  use logirecon::parser::provider::SheetProvider;
///
///  let mut provider = SheetProvider::new(headers, primary);
///  provider
///      .update_headers(headers_mapping)
///      .add_sheets(path, sheet);
///  for df_res in provider.try_get_dataframes() {
///       println!("{}", df_res?);
/// }
/// ```
#[derive(Debug)]
pub struct SheetProvider {
    /// 默认表头和表头别名
    headers: HashMap<String, String>,
    /// 路径和表单名
    sheets: Vec<(PathBuf, String)>,
    /// 使用哪一列确定数据的行数, 此选项不影响最终提供的Excel数据
    primary: String,
}

impl SheetProvider {
    /// 创建一个提取器
    pub fn new<I, S>(default_headers: I, primary_key: S) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let iter = default_headers.into_iter().map(|t| {
            let h = t.into();
            (h.clone(), h)
        });
        Self {
            headers: HashMap::from_iter(iter),
            sheets: vec![],
            primary: primary_key.into(),
        }
    }

    /// 返回默认表头
    pub fn default_headers(&self) -> Vec<&String> {
        self.headers.keys().collect()
    }

    /// 返回当前表头
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// 返回当前主键
    pub fn primary(&self) -> &String {
        &self.primary
    }

    /// 返回当前sheets
    pub fn sheets(&self) -> &Vec<(PathBuf, String)> {
        &self.sheets
    }

    /// 更新表头
    ///
    /// 只更新默认表头中已有的键
    pub fn update_headers<S, I>(&mut self, headers: I) -> &mut Self
    where
        S: Into<String>,
        I: IntoIterator<Item = (S, S)>,
    {
        for (k, header) in headers.into_iter().map(|(k, v)| (k.into(), v.into())) {
            if self.headers.contains_key(&k) {
                if let Some(v) = self.headers.get_mut(&k) {
                    *v = header;
                }
            }
        }
        self
    }

    /// 清空当前 sheets
    pub fn clear_sheets(&mut self) -> &mut Self {
        self.sheets.clear();
        self
    }

    /// 添加sheet
    pub fn add_sheets(&mut self, path: impl Into<PathBuf>, sheet: impl Into<String>) -> &mut Self {
        self.sheets.push((path.into(), sheet.into()));
        self
    }

    /// 获取数据
    pub fn try_get_dataframes(&self) -> impl Iterator<Item = Result<DataFrame>> {
        let headers = self.headers.values();
        let primary = self.primary();

        self.sheets().iter().map(move |(path, sheet)| {
            let opts = ExcelReadOptions::default()
                .with_headers(headers.clone())
                .with_primary(primary)
                .with_path(path)
                .with_sheet(sheet);
            let reader = opts.try_into_reader()?;
            reader.finish()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let default_headers = ["运单号", "货件单号"];
        let primary_key = "序号";
        let mut provider = SheetProvider::new(default_headers, primary_key);
        provider
            .update_headers([("货件单号", "订单号")])
            .add_sheets("data/test.xlsx", "Sheet1");
        println!("{:?}", provider);
    }
}
