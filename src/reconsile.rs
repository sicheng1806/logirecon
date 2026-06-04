use std::collections::HashMap;
use std::path::PathBuf;

use crate::{DataFrame, Error, Result, Schema};

pub enum ReconsileType {
    Numberic(f64),
    Exact,
}

/// 对账选项
///
/// 核对两张类型一致的表格
///
/// 类型通过[Schema]约定
/// 可添加核对字段和核对类型
pub struct ReconsileOptions {
    schema: Option<Schema>,
    resonsile_fields: HashMap<String, ReconsileType>,
}

/// 对账器
pub struct Reconsiler {
    resonsile_fields: HashMap<String, ReconsileType>,
    left: (String, DataFrame),
    right: (String, DataFrame),
    result: DataFrame,
}

/// Excel导出选项
pub struct ExcelExportOptions {}

impl ReconsileOptions {
    pub fn with_reconsile(&mut self, column: &str, rtype: ReconsileType) -> &mut Self {
        todo!()
    }

    pub fn with_schema(&mut self, schema: Schema) -> &mut Self {
        todo!()
    }

    pub fn with_compare(&mut self, name: impl Into<String>, df: DataFrame) -> &mut Self {
        todo!()
    }

    pub fn try_into_reconsiler(self) -> Result<Reconsiler> {
        todo!()
    }
}

impl Reconsiler {
    /// 对账, 获得数据框比对结果
    pub fn reconsile(&mut self) -> Result<&mut Self> {
        todo!()
    }
    /// 返回长格式比对结果
    pub fn get_long_result(&self) -> Result<DataFrame> {
        todo!()
    }

    /// 返回更易读的宽格式结果
    pub fn get_width_result(&self) -> Result<DataFrame> {
        todo!()
    }

    /// 导出到结果到excel
    pub fn to_excel(&self, path: impl Into<PathBuf>, opts: ExcelExportOptions) -> Result<PathBuf> {
        todo!()
    }
}
