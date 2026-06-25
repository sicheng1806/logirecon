mod common;
use common::*;
use logirecon::prelude::runner::*;
use logirecon::prelude::*;

#[test]
fn test_runner() {
    let wb = Template {
        parse_config: ParseConfig::WB(WBParseConfig::default()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_WB.into(),
        }],
    };
    let grt = Template {
        parse_config: ParseConfig::GRT(WBParseConfig::grt()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_GRT.into(),
        }],
    };
    let tsbg = Template {
        parse_config: ParseConfig::TS(TSParseConfig::default()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_TSBG.into(),
        }],
    };
    let tsyf = Template {
        parse_config: ParseConfig::TS(TSParseConfig::default()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_TSYF.into(),
        }],
    };
    let ddd = Template {
        parse_config: ParseConfig::DDD(DDDParseConfig::default()),
        sources: vec![ReadConfig::ExcelFilePath {
            path: PATH_BILLS.clone(),
            name: SHEET_DDD.into(),
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
            }],
        }
    };
    let templates = vec![wb, grt, ddd, tsbg, tsyf, headway];
    let (freight_reconciler, customs_reconciler) = get_reconciler(templates).unwrap();
    let freight = freight_reconciler.get_long_result().unwrap();
    let customs = customs_reconciler.get_long_result().unwrap();
    let stasis_result = stasis_freight_and_customs(freight, customs).unwrap();
    println!("对账分析结果:\n{}", stasis_result);
}
