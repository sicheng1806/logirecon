mod common;

use logirecon::reader::{ExcelError, ExcelReader};

use crate::common::*;

#[test]
fn test_read() -> Result<(), ExcelError> {
    let path = PATH_BILLS.clone();
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
        let df = ExcelReader::new(headers)
            .load_worksheet(&path, sheet)?
            .primary("序号")
            .read()?;

        println!("{}", df);
    }

    let path = PATH_HEADWAY.clone();
    for (&sheet, headers) in [SHEET_HEADWAY_2026, SHEET_HEADWAY_2025]
        .iter()
        .zip([HEADERS_HEADWAY_2026.to_vec(), HEADERS_HEADWAY_2025.to_vec()])
    {
        let df = ExcelReader::new(headers)
            .load_worksheet(&path, sheet)?
            .primary("序号")
            .read()?;
        println!("{}", df);
    }
    Ok(())
}

#[test]
fn test_read_headway() {
    for _i in 0..10 {
        let _df = ExcelReader::new(HEADERS_HEADWAY_2025)
            .load_worksheet(PATH_HEADWAY.clone(), SHEET_HEADWAY_2025)
            .unwrap()
            .primary("序号")
            .read()
            .unwrap();
    }
}
