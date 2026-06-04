use std::{collections::HashSet, path::PathBuf};

/// Excel读取选项
#[derive(Debug, Clone)]
pub struct ExcelReadOptions {
    pub headers: HashSet<String>,
    pub path: Option<PathBuf>,
    pub sheet: Option<String>,
    pub primary_key: Option<String>,
}

impl Default for ExcelReadOptions {
    fn default() -> Self {
        Self {
            headers: HashSet::new(),
            path: None,
            sheet: None,
            primary_key: None,
        }
    }
}

impl ExcelReadOptions {
    pub fn with_headers(mut self, iter: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.headers.extend(iter.into_iter().map(|t| t.into()));
        self
    }

    pub fn with_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_sheet<P: Into<String>>(mut self, name: P) -> Self {
        self.sheet = Some(name.into());
        self
    }

    pub fn with_primary<P: Into<String>>(mut self, name: P) -> Self {
        self.primary_key = Some(name.into());
        self
    }
}