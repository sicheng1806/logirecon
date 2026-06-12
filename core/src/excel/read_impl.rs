use std::collections::{HashMap, HashSet};

use calamine::{Data, DataType, Range};
use polars::{
    datatypes::{DataType as PlDataType, PlSmallStr},
    series::Series,
};

use crate::{Error, Result};

pub fn get_headers_and_datarange(
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

pub fn data_to_series(
    name: impl Into<PlSmallStr>,
    datas: impl IntoIterator<Item = Data>,
) -> Result<Series> {
    let datas: Vec<Data> = datas.into_iter().collect();
    if datas.is_empty() {
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
