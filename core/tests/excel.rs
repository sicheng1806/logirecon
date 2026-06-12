use logirecon_core as logirecon;

use logirecon::parse::provider::SheetProvider;
use logirecon::{ExcelReadOptions, Result};

mod common;
use common::*;

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

#[test]
fn test_provider() -> Result<()> {
    let path = PATH_BILLS;
    let sheet = SHEET_WB;
    let headers = [
        "日期",
        "运单号",
        "货运单号",
        "物流中心编码",
        "件数",
        "收费重",
        "单价",
        "费用类型",
        "金额",
    ];
    let mut provider = SheetProvider::new(headers, "序号");
    provider
        .update_headers(headers.into_iter().zip(HEADERS_WB))
        .add_sheets(path, sheet);
    for df_res in provider.try_get_dataframes() {
        println!("{}", df_res?);
    }
    Ok(())
}

#[test]
fn test_write() -> Result<()> {
    use chrono::prelude::*;
    use polars::prelude::*;
    let df: DataFrame = df!(
        "String" => &["North", "South", "East", "West"],
        "Integer" => &[1, 2, 3, 4],
        "Float" => &[4.0, 5.0, 6.0, 7.0],
        "Time" => &[
            NaiveTime::from_hms_milli_opt(2, 59, 3, 456).unwrap(),
            NaiveTime::from_hms_milli_opt(2, 59, 3, 456).unwrap(),
            NaiveTime::from_hms_milli_opt(2, 59, 3, 456).unwrap(),
            NaiveTime::from_hms_milli_opt(2, 59, 3, 456).unwrap(),
            ],
        "Date" => &[
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 3).unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 4).unwrap(),
            ],
        "Datetime" => &[
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap().and_hms_opt(1, 0, 0).unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 2).unwrap().and_hms_opt(2, 0, 0).unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 3).unwrap().and_hms_opt(3, 0, 0).unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 4).unwrap().and_hms_opt(4, 0, 0).unwrap(),
        ],
    )?;

    use polars_excel_writer::PolarsExcelWriter;
    // Create a new Excel writer.
    let mut excel_writer = PolarsExcelWriter::new();
    excel_writer.set_worksheet_name("Sheet Name")?;

    // Write the dataframe to Excel.
    excel_writer.write_dataframe(&df)?;

    // Save the file to disk.
    excel_writer.save("data/test/dataframe.xlsx")?;
    Ok(())
}
