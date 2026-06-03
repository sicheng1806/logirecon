use crate::{Error, Result, excel::ExcelReadOptions};
use calamine::{Data, DataType, Range, Reader};
use log::debug;
use polars::{
    datatypes::PlSmallStr,
    frame::{DataFrame, column::Column},
    series::Series,
};
use std::collections::{HashMap, HashSet};

type PlDataType = polars::datatypes::DataType;

/// Excel 表格读取器
#[derive(Debug)]
pub struct ExcelReader {
    range: Range<Data>,
    headers: HashMap<String, u32>,
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

fn data_to_series(
    name: impl Into<PlSmallStr>,
    datas: impl IntoIterator<Item = Data>,
) -> Result<Series> {
    let datas: Vec<Data> = datas.into_iter().collect();
    if datas.len() == 0 {
        return Ok(Series::new_empty(name.into(), &PlDataType::String));
    }
    // 确定列类型
    let mut only_one_type = true; // 是否为String类型
    let mut dtype = None;
    for data in &datas {
        let data_type = get_dtype(data);
        if data_type == PlDataType::Null {
            continue;
        }
        if dtype.is_none() {
            dtype = Some(data_type)
        } else if Some(data_type) != dtype {
            only_one_type = false;
        }
    }
    // 生成给定类型的Series
    if let Some(dtype) = dtype {
        let dtype = if only_one_type {
            dtype
        } else {
            PlDataType::String
        };
        Ok(data_to_series_with_opts(datas, dtype).with_name(name.into()))
    } else {
        // first_type = None, 说明全部为空
        Ok(Series::new_empty(name.into(), &PlDataType::String))
    }
}

fn get_dtype(data: &Data) -> PlDataType {
    match data {
        Data::Float(_) => PlDataType::Float64,
        Data::Int(_) => PlDataType::Int64,
        Data::Bool(_) => PlDataType::Boolean,
        Data::DateTime(_) => PlDataType::Date,
        Data::Empty => PlDataType::Null,
        Data::String(v) => {
            if v.is_empty() {
                PlDataType::Null
            } else {
                PlDataType::String
            }
        }
        _ => PlDataType::String,
    }
}

fn data_to_series_with_opts(datas: Vec<Data>, dtype: PlDataType) -> Series {
    let s: Series = match dtype {
        PlDataType::Float64 => datas.into_iter().map(|t| t.as_f64()).collect(),
        PlDataType::Int64 => datas.into_iter().map(|t| t.as_i64()).collect(),
        PlDataType::Date => datas
            .into_iter()
            .map(|t| {
                if let Some(date) = t.as_date() {
                    date.to_string()
                } else {
                    "".into()
                }
            })
            .collect(),
        _ => datas.into_iter().map(|t| t.to_string()).collect(),
    };
    if dtype == PlDataType::Date {
        s.cast(&PlDataType::Date).unwrap()
    } else {
        s
    }
}

fn get_headers_and_datarange(
    range: Range<Data>,
    headers: impl IntoIterator<Item = String>,
    primary_key: String,
) -> Result<(Range<Data>, HashMap<String, u32>)> {
    let mut headers: HashSet<String> = HashSet::from_iter(headers);
    headers.insert(primary_key.clone());
    let mut headers_mapping: HashMap<String, u32> = HashMap::new();
    let raw_start = range.start().ok_or("range.start() is none")?;
    let raw_end = range.end().ok_or("range.end() is none")?;
    let mut headers_row_idx: Option<u32> = None;

    // 查找表头行，完全严格匹配
    'main: for (row_num, row) in range.rows().enumerate() {
        headers_mapping.clear();
        for (col_num, data) in row.iter().enumerate() {
            let data_str = data.to_string();
            if headers.contains(&data_str) {
                let col_idx = col_num as u32 + raw_start.1;
                headers_mapping.insert(data_str, col_idx);
            }
            // 退出点
            if headers.len() <= headers_mapping.len() {
                headers_row_idx = Some(row_num as u32 + raw_start.0);
                break 'main;
            }
        }
    }
    if headers_row_idx.is_none() {
        return Err(Error::IO("未找到所有的表头".into()));
    }
    // 根据主键列确定表格区域
    let _col_idx = headers_mapping.get(&primary_key).unwrap();
    let _start = (headers_row_idx.unwrap(), *_col_idx);
    let _end = (raw_end.0, *_col_idx);
    let mut data_type = None;
    let mut end_idx = _start.0;
    for (row_num, _, data) in range.range(_start, _end).cells() {
        if row_num == 0 {
            continue;
        }
        if row_num == 1 {
            data_type = Some(get_dtype(data));
        }
        if Some(get_dtype(data)) != data_type {
            break;
        }
        end_idx += 1;
    }
    let headers_col_idx_min = headers_mapping.values().min().unwrap();
    let headers_col_idx_max = headers_mapping.values().max().unwrap();
    let start = (headers_row_idx.unwrap(), *headers_col_idx_min);
    let end = (end_idx, *headers_col_idx_max);
    let data_range = range.range(start, end);
    Ok((data_range, headers_mapping))
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    use super::*;

    #[test]
    fn test_new() -> Result<()> {
        let range = Range::new((1, 1), (5, 5));
        let reader = ExcelReader::new(
            range,
            HEADERS_HEADWAY_2026
                .iter()
                .enumerate()
                .map(|(i, &t)| (t, i as u32))
                .collect::<Vec<(&str, u32)>>(),
        );
        println!("{:?}", reader);
        Ok(())
    }

    #[test]
    fn test_get_headers_and_datarange() -> Result<()> {
        use calamine::{Cell, Data};
        let cells: Vec<Cell<Data>> = vec![
            // 干扰项
            Cell::new((0, 1), 1.0.into()),
            Cell::new((0, 2), 2.0.into()),
            // 表头
            Cell::new((1, 0), "PK".into()),
            Cell::new((1, 1), "Other".into()),
            // PK列
            Cell::new((2, 0), 1.into()),
            Cell::new((3, 0), 1.into()),
            Cell::new((4, 0), 1.into()),
            Cell::new((5, 0), "1".into()),
        ];
        let range = Range::from_sparse(cells);
        let headers = ["Other".to_string()];
        let primary_key = "PK".to_string();

        let (data_range, headers_mapping) = get_headers_and_datarange(range, headers, primary_key)?;
        assert_eq!(data_range.height(), 4); // 1-4
        assert_eq!(data_range.width(), 2);
        assert_eq!(data_range.headers().unwrap(), vec!["PK", "Other"]);
        assert_eq!(
            headers_mapping.values().map(|t| *t).collect::<Vec<u32>>(),
            vec![0, 1]
        );

        Ok(())
    }

    #[test]
    fn test_read() -> Result<()> {
        let path = PATH_BILLS;
        for (&sheet, headers) in [SHEET_WB, SHEET_GRT, SHEET_DDD, SHEET_TSYF, SHEET_TSBG]
            .iter()
            .zip([
                HEADERS_WB.to_vec(),
                HEADERS_GRT.to_vec(),
                HEADERS_DDD.to_vec(),
                HEADERS_TSYF.to_vec(),
                HEADERS_TSBG.to_vec(),
            ])
        {
            let df = ExcelReadOptions::default()
                .with_headers(headers)
                .with_path(path)
                .with_sheet(sheet)
                .with_primary("序号")
                .try_into_reader()?
                .finish()?;
            println!("{}", df);
        }

        let path = PATH_HEADWAY;
        for (&sheet, headers) in [SHEET_HEADWAY_2026, SHEET_HEADWAY_2025]
            .iter()
            .zip([HEADERS_HEADWAY_2026.to_vec(), HEADERS_HEADWAY_2025.to_vec()])
        {
            let df = ExcelReadOptions::default()
                .with_headers(headers)
                .with_path(path)
                .with_sheet(sheet)
                .with_primary("序号")
                .try_into_reader()?
                .finish()?;
            println!("{}", df);
        }
        Ok(())
    }
}
