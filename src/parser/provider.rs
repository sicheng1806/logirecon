use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct SheetProvider {
    headers: HashMap<String, String>,
    sheets: Vec<(PathBuf, String)>,
    primary: String,
    default_headers: Vec<String>,
}

impl SheetProvider {
    pub fn new<I, S>(default_headers: I, primary_key: S) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let default_headers: Vec<String> = default_headers.into_iter().map(|t| t.into()).collect();
        Self {
            headers: HashMap::from_iter(
                default_headers.iter().map(|t| (t.to_owned(), t.to_owned())),
            ),
            sheets: vec![],
            primary: primary_key.into(),
            default_headers: default_headers,
        }
    }

    /// 返回默认表头
    pub fn default_headers(&self) -> &Vec<String> {
        &self.default_headers
    }
    /// 返回当前表头
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
    /// 返回当前主键
    pub fn primary(&self) -> &String {
        &self.primary
    }
    /// 更新表头
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

    /// 返回当前sheets
    pub fn sheets(&self) -> &Vec<(PathBuf, String)> {
        &self.sheets
    }

    /// 清空当前sheets
    pub fn clear_sheets(&mut self) -> &mut Self {
        self.sheets.clear();
        self
    }

    /// 添加sheet
    pub fn add_sheets<P, S>(&mut self, path: P, sheet: S) -> &mut Self
    where
        P: Into<PathBuf>,
        S: Into<String>,
    {
        self.sheets.push((path.into(), sheet.into()));
        self
    }
}
