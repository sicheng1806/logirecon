use log::debug;
use polars::frame::DataFrame;
use polars::frame::column::Column;
use std::collections::HashMap;

use calamine::{Data, Range, Reader};

use super::options::ExcelReadOptions;
use super::read_impl::{data_to_series, get_headers_and_datarange};
use crate::Result;

/// Excel 表格读取器
#[derive(Debug)]
pub struct ExcelReader {
    range: Range<Data>,
    headers: HashMap<String, u32>,
}
impl ExcelReader {
    pub fn new<I, P>(range: Range<Data>, headers: I) -> Self
    where
        I: IntoIterator<Item = (P, u32)>,
        P: Into<String>,
    {
        Self {
            range,
            headers: HashMap::from_iter(headers.into_iter().map(|(name, idx)| (name.into(), idx))),
        }
    }
}

impl ExcelReadOptions {
    pub fn try_into_reader(self) -> Result<ExcelReader> {
        let path = self.path.ok_or("实现错误: 缺少文件路径参数")?;
        let headers = self.headers;
        let sheet = self.sheet.ok_or("实现错误: 缺少表单名称")?;
        let primary_key = self.primary_key.ok_or("实现错误: 缺少用于确定区域的主键")?;

        debug!("读取表格: {:?} {}", path.as_os_str(), &sheet);
        //
        let mut wb = calamine::open_workbook_auto(path)?;
        let range = wb.worksheet_range(&sheet)?;
        //
        let (data_range, headers_cols) = get_headers_and_datarange(range, headers, primary_key)?;
        let reader = ExcelReader::new(data_range, headers_cols);
        Ok(reader)
    }
}

impl ExcelReader {
    pub fn finish(self) -> Result<DataFrame> {
        // 按列读取表格数据, 表格首行为表格头
        let headers: HashMap<u32, String> =
            HashMap::from_iter(self.headers.into_iter().map(|(k, v)| (v, k)));
        let range = self.range;

        debug!("以表头: {:?} 读取数据", headers.values());

        let mut columns_data: HashMap<String, Vec<Data>> = HashMap::new();
        for name in headers.values() {
            columns_data.insert(name.to_owned(), vec![]);
        }
        // 填充 columns_data
        let start = range.start().unwrap();
        for (row_num, row) in range.rows().enumerate() {
            if row_num == 0 {
                continue;
            }
            for (col_num, data) in row.iter().enumerate() {
                let col_idx = col_num as u32 + start.1;
                if headers.contains_key(&col_idx) {
                    let name = headers.get(&col_idx).unwrap();
                    if let Some(v) = columns_data.get_mut(name) {
                        v.push(data.to_owned());
                    }
                }
            }
        }

        // 将列转换为Series
        let mut columns: Vec<Column> = vec![];
        for (name, data) in columns_data {
            let col = data_to_series(name, data)?;
            columns.push(col.into());
        }

        let df = DataFrame::new_infer_height(columns)?;
        debug!("表格读取成功: {:?}", df.schema());
        Ok(df)
    }
}
