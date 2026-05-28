use std::{fs::File, io::BufReader, mem::discriminant};

use crate::Result;
use crate::error::Error;
use calamine::{Data, Range, Reader, Sheets};
use polars::{
    datatypes::{AnyValue, TimeUnit},
    frame::{DataFrame, column::Column},
};

fn data_to_anyvalue(data: &'_ Data) -> Result<AnyValue<'_>> {
    match *data {
        Data::Int(value) => Ok(AnyValue::Int64(value)),
        Data::Float(value) => Ok(AnyValue::Float64(value)),
        Data::String(ref value) => Ok(AnyValue::String(value)),
        Data::Bool(value) => Ok(AnyValue::Boolean(value)),
        Data::DateTime(value) => {
            if let Some(dt) = value.as_datetime() {
                Ok(AnyValue::Datetime(
                    dt.and_utc().timestamp_millis(),
                    TimeUnit::Milliseconds,
                    None,
                ))
            } else {
                Err(Error::ParseError("Excel日期解析错误".into()))
            }
        }
        Data::DateTimeIso(ref value) => Ok(AnyValue::String(value)),
        Data::DurationIso(ref value) => Ok(AnyValue::String(value)),
        Data::Error(ref e) => Err(Error::ParseError(format!("Cell Error: {e}"))),
        Data::Empty => Ok(AnyValue::Null),
    }
}

pub fn range_to_dataframe(rng: Range<Data>) -> Result<DataFrame> {
    let header = rng
        .headers()
        .ok_or("Range中缺少Header")
        .map_err(|e| Error::NotFoundError(e.to_string()))?;
    let mut columns_data: Vec<Vec<AnyValue>> = vec![];
    for _ in 0..header.len() {
        columns_data.push(vec![]);
    }
    for (nrow, row) in rng.rows().enumerate() {
        if nrow == 0 {
            continue;
        }
        for (ncol, data) in row.iter().enumerate() {
            columns_data[ncol].push(data_to_anyvalue(data)?);
        }
    }
    let columns: Vec<Column> = columns_data
        .iter()
        .enumerate()
        .map(|(ncol, cols)| Column::new(header[ncol].as_str().into(), cols))
        .collect();
    let df = DataFrame::new_infer_height(columns)?;
    Ok(df)
}

pub fn read_excel(
    workbook: &mut Sheets<BufReader<File>>,
    sheet_name: &str,
    test_header: &Vec<&str>,
    primary_key: &str,
) -> Result<DataFrame> {
    let sheet = workbook.worksheet_range(sheet_name)?;
    let mut headers: Vec<&str> = vec![primary_key];
    test_header.iter().for_each(|&h| {
        if h != primary_key {
            headers.push(h)
        }
    });
    // 获取表头对应行索引
    let start = sheet.start().ok_or("区域应该存在开头单元格的")?;
    let end = sheet.end().ok_or("区域应该存在结束单元格")?;
    let mut header_ridx: Option<u32> = None;
    let mut header_mapping: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();
    // 遍历行以获取头部
    'main: for (nrow, row) in sheet.rows().enumerate() {
        // 取主键对应行索引为表头行索引
        for (ncol, cell) in row.iter().enumerate() {
            let cell_str = cell.to_string();
            if headers.contains(&cell_str.as_str()) {
                if &cell_str == primary_key {
                    header_ridx = Some(nrow as u32 + start.0);
                }
                header_mapping.insert(cell_str, ncol as u32 + start.1);
            }
            // 若匹配完成，退出循环
            if header_mapping.len() == headers.len() {
                break 'main;
            }
        }
        // 进入下次匹配
        if !header_mapping.is_empty() {
            header_mapping.clear();
        }
    }

    if header_mapping.len() != headers.len() {
        return Err(Error::NotFoundError("未找到表头行".into()));
    }

    if let Some(header_ridx) = header_ridx {
        // 有header_ridx, header_mapping, start, end
        // 根据主键列索引和Data枚举类型匹配，判断数据体截至行索引
        let &pk_cidx = header_mapping
            .get(primary_key)
            .ok_or("不会触发: 未找到主键列索引")?;
        // 迭代header_ridx + 1 到 end.0 的主键列，根据首个匹配数据类型确定数据区域
        let mut first_cell: Option<&Data> = None;
        let mut end_ridx: u32 = header_ridx;
        for (nrow, row) in sheet
            .range((header_ridx + 1, pk_cidx), (end.0, pk_cidx))
            .rows()
            .enumerate()
        {
            let cell = &row[0];
            if nrow == 0 {
                first_cell = Some(cell);
            }
            if discriminant(cell) != discriminant(first_cell.unwrap()) {
                break;
            }
            end_ridx += 1;
        }
        let rng_start = (
            header_ridx,
            *header_mapping
                .values()
                .min()
                .ok_or("表头列索引应该有最小值")?,
        );
        let rng_end = (
            end_ridx,
            *header_mapping
                .values()
                .max()
                .ok_or("表头列索引应该有最大值")?,
        );
        let rng = sheet.range(rng_start, rng_end);
        let df = range_to_dataframe(rng)?;
        Ok(df)
    } else {
        Err(Error::NotFoundError(
            "理论上不会触发此错误：未找到表头行索引".into(),
        ))
    }
}

#[cfg(test)]
mod polars_excel {
    use std::mem::discriminant;

    use calamine::Reader;

    use super::*;

    #[test]
    fn test_xlsx_to_dataframe() {
        let mut workbook = calamine::open_workbook_auto("data/test/iris.xlsx").unwrap();
        let sheet = workbook.worksheet_range("Sheet1").unwrap();
        println!(
            "rows = {}, columns = {}",
            sheet.get_size().0,
            sheet.get_size().1
        );
        // 构建表格
        let df = range_to_dataframe(sheet).unwrap();
        println!("{}", df.head(None));
    }

    #[test]
    fn test_read_excel() -> Result<()> {
        let path = "data/test/物流账单.xlsx";
        let sheet_name = "万邦2604";
        let mut workbook = calamine::open_workbook_auto(path)?;
        let test_header = vec![
            "序号",
            "日期",
            "运单号",
            "订单号",
            "渠道",
            "发往国家",
            "仓库编码",
            "件数",
            "收费重",
            "单价",
            "费用类型",
            "金额",
        ];

        let df = read_excel(&mut workbook, sheet_name, &test_header, "序号")?;
        assert_eq!(df.shape(), (30, 12));
        println!("{:?}", df.schema());
        for h in test_header {
            assert!(df.schema().contains(h));
        }
        println!("{}", df.head(None));
        // println!("{}", df.tail(None));
        Ok(())
    }

    #[test]
    fn test_discriminat() {
        assert_eq!(
            discriminant(&Data::Bool(true)),
            discriminant(&Data::Bool(true))
        );
        assert_eq!(
            discriminant(&Data::Bool(true)),
            discriminant(&Data::Bool(false))
        );
        assert!(discriminant(&Data::Bool(true)) != discriminant(&Data::Empty));
    }
}
