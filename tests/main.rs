mod common;
use common::*;

use std::{collections::HashMap, path::PathBuf};

pub struct UserData {
    pub parser_type: ParserType,
    pub headers: HashMap<String, String>,
    pub sheets: Vec<(PathBuf, String)>,
    pub primary: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserType {
    WB,
    Headway,
}

#[test]
fn test_main() -> Result<(), Box<dyn std::error::Error>> {
    use logirecon::reconsile::{CUSTOMS_RECONSILE_COLUMNS, FREIGHT_RECONSILE_COLUMNS};
    use logirecon::{DataRepo, HeadwayParser, Parse, ReconsileOption, WBParser};
    use polars_excel_writer::PolarsExcelWriter;
    // 模拟UI返回的用户输入结构体
    let bills: Vec<UserData> = vec![UserData {
        parser_type: ParserType::WB,
        headers: HashMap::from_iter(
            WBParser::DEFAULT_HEADERS.map(|t| (t.to_string(), t.to_string())),
        ),
        sheets: vec![(PATH_BILLS.into(), SHEET_WB.to_string())],
        primary: "序号".to_string(),
    }];

    let shipments = vec![UserData {
        parser_type: ParserType::Headway,
        headers: HashMap::from_iter(HeadwayParser::DEFAULT_HEADERS.map(|t| {
            if t != "报关费" {
                (t.to_string(), t.to_string())
            } else {
                (t.to_string(), "报关或其他费".to_string())
            }
        })),
        sheets: vec![(PATH_HEADWAY.into(), SHEET_HEADWAY_2026.to_string())],
        primary: "序号".to_string(),
    }];

    // parse
    let bills = bills.into_iter().filter_map(|data| {
        let mut parser = match data.parser_type {
            ParserType::WB => WBParser::default(),
            _ => return None,
        };
        parser
            .provider_mut()
            .update_headers(data.headers)
            .with_primary(data.primary);
        for (path, sheet) in data.sheets {
            parser.provider_mut().add_sheets(path, sheet);
        }
        parser.parse().ok()
    });
    let shipments = shipments.into_iter().filter_map(|data| {
        let mut parser = match data.parser_type {
            ParserType::Headway => HeadwayParser::default(),
            _ => return None,
        };
        parser
            .provider_mut()
            .update_headers(data.headers)
            .with_primary(data.primary);
        for (path, sheet) in data.sheets {
            parser.provider_mut().add_sheets(path, sheet);
        }
        parser.parse().ok()
    });
    // user input 解析
    let repo = DataRepo::new(bills, shipments)?;
    // 获取运单和报关单的差异分析报表
    let (freight_bill, freight_self) = repo.get_freight()?;
    let freight_report = ReconsileOption::new_with_columns(FREIGHT_RECONSILE_COLUMNS)
        .left(freight_bill, "物流")
        .right(freight_self, "我方")
        .try_into_reconsiler()?
        .build_result()?
        .get_long_result()?;
    let (customs_bill, customs_self) = repo.get_customs()?;
    let customs_report = ReconsileOption::new_with_columns(CUSTOMS_RECONSILE_COLUMNS)
        .left(customs_bill, "物流")
        .right(customs_self, "我方")
        .try_into_reconsiler()?
        .build_result()?
        .get_long_result()?;
    // 导出为excel
    let mut wb = PolarsExcelWriter::new();
    wb.set_worksheet_name("运费比对结果")?;
    wb.write_dataframe(&freight_report)?;
    wb.add_worksheet();
    wb.set_worksheet_name("报关费比对结果")?;
    wb.write_dataframe(&customs_report)?;
    wb.save("data/test/output.xlsx")?;
    Ok(())
}
