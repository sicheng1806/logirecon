mod common;
use common::*;
use logirecon::prelude::runner::*;
use logirecon::prelude::*;

#[test]
fn test_runner() {
    let wb = Template {
        parse_config: ParseConfig::Wb(WBParseConfig::default()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_WB.into(),
            primary: "序号".into(),
        }],
    };
    let grt = Template {
        parse_config: ParseConfig::Wb(WBParseConfig::grt()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_GRT.into(),
            primary: "序号".into(),
        }],
    };
    let jm = Template {
        parse_config: ParseConfig::Wb(WBParseConfig::jm()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_JM.into(),
            primary: "序号".into(),
        }],
    };
    let tsbg = Template {
        parse_config: ParseConfig::Ts(TsParseConfig::default()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_TSBG.into(),
            primary: "序号".into(),
        }],
    };
    let tsyf = Template {
        parse_config: ParseConfig::Ts(TsParseConfig::default()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_TSYF.into(),
            primary: "序号".into(),
        }],
    };
    let ddd = Template {
        parse_config: ParseConfig::Ddd(DddParseConfig::default()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_DDD.into(),
            primary: "序号".into(),
        }],
    };
    let jyd = Template {
        parse_config: ParseConfig::Jyd(JydParseConfig::default()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_JYD.into(),
            primary: "签入日期".into(),
        }],
    };
    let headway = {
        let mut config = HeadwayParseConfig::default();
        config.year = 2026;
        config.headers.customs_fee = "报关或其他费".into();
        Template {
            parse_config: ParseConfig::Headway(config),
            sources: vec![ReadConfig::ExcelFilePath {
                path: PATH_HEADWAY.clone(),
                name: SHEET_HEADWAY_2026.into(),
                primary: "序号".into(),
            }],
        }
    };
    let templates = vec![wb, grt, jm, ddd, tsbg, tsyf, jyd, headway];
    let (freight_reconciler, customs_reconciler) = get_reconciler(templates).unwrap();
    let freight = freight_reconciler.get_long_result().unwrap();
    let customs = customs_reconciler.get_long_result().unwrap();
    let stasis_result = stasis_freight_and_customs(freight, customs).unwrap();
    println!("对账分析结果:\n{}", stasis_result);
}
