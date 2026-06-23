mod common;
use common::*;
use logirecon::{
    parser::{
        AsHeaders, DDDParseConfig, DDDParser, HeadwayParseConfig, HeadwayParser, Parse,
        TSParseConfig, TSParser, WBParseConfig, WBParser,
    },
    reader::ExcelReader,
    validate::IntoValidated,
};

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[test]
fn test_wb() -> Result<()> {
    let config = WBParseConfig::default();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_WB)?
        .read()?;
    let bill = WBParser::parse(data, config)?;
    println!("{}", bill.into_validated()?);
    Ok(())
}

#[test]
fn test_grt() -> Result<()> {
    let mut config = WBParseConfig::default();
    config.headers.shipment_no = "扩展单号".into();
    config.headers.warehouse_code = "地址编码".into();
    config.forwarder = "国润通".into();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_GRT)?
        .read()?;
    let bill = WBParser::parse(data, config)?;
    println!("{}", bill.into_validated()?);
    Ok(())
}

#[test]
fn test_ts() -> Result<()> {
    let config = TSParseConfig::default();
    let data_bg = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_TSBG)?
        .read()?;
    let data_yf = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_TSYF)?
        .read()?;
    let bills = (
        TSParser::parse(data_bg, config.clone())?,
        TSParser::parse(data_yf, config)?,
    );
    println!("{}", bills.0.into_validated()?);
    println!("{}", bills.1.into_validated()?);
    Ok(())
}

#[test]
fn test_ddd() -> Result<()> {
    let config = DDDParseConfig::default();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_DDD)?
        .read()?;
    let bill = DDDParser::parse(data, config)?;
    println!("{}", bill.into_validated()?);
    Ok(())
}

#[test]
fn test_get_shipment() -> Result<()> {
    let mut config = HeadwayParseConfig::default();
    config.headers.customs_fee = "报关或其他费".into();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .primary("序号")
        .load_worksheet(PATH_HEADWAY.clone(), SHEET_HEADWAY_2026)?
        .read()?;
    let shipment = HeadwayParser::parse(data, config)?;
    println!("{}", shipment.into_validated()?);
    Ok(())
}
