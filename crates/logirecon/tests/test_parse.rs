mod common;
use common::*;
use logirecon::{
    parser::{
        AsHeaders, DddParseConfig, DddParser, HeadwayParseConfig, HeadwayParser, JydParseConfig,
        JydParser, Parse, TsParseConfig, TsParser, WBParseConfig, WbParser,
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
    let bill = WbParser::parse(data, config)?;
    println!("{}", bill.into_validated()?);
    Ok(())
}

#[test]
fn test_grt() -> Result<()> {
    let config = WBParseConfig::grt();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_GRT)?
        .read()?;
    let bill = WbParser::parse(data, config)?;
    println!("{}", bill.into_validated()?);
    Ok(())
}

#[test]
fn test_jm() -> Result<()> {
    let config = WBParseConfig::jm();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_JM)?
        .read()?;
    let bill = WbParser::parse(data, config)?;
    println!("{}", bill.into_validated()?);
    Ok(())
}

#[test]
fn test_ts() -> Result<()> {
    let config = TsParseConfig::default();
    let data_bg = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_TSBG)?
        .read()?;
    let data_yf = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_TSYF)?
        .read()?;
    let bills = (
        TsParser::parse(data_bg, config.clone())?,
        TsParser::parse(data_yf, config)?,
    );
    println!("{}", bills.0.into_validated()?);
    println!("{}", bills.1.into_validated()?);
    Ok(())
}

#[test]
fn test_ddd() -> Result<()> {
    let config = DddParseConfig::default();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_DDD)?
        .read()?;
    let bill = DddParser::parse(data, config)?;
    println!("{}", bill.into_validated()?);
    Ok(())
}

#[test]
fn test_jyd() -> Result<()> {
    let config = JydParseConfig::default();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_JYD)?
        .read()?;
    let bill = JydParser::parse(data, config)?;
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
