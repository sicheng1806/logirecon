mod common;
use common::*;
use logirecon::{
    parser::{AsHeaders, HeadwayParseConfig, HeadwayParser, Parse, WBParseConfig, WBParser},
    process::Processor,
    reader::ExcelReader,
};

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[test]
fn test_process() -> Result<()> {
    let config = WBParseConfig::default();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_BILLS.clone(), SHEET_WB)?
        .read()?;
    let bill = WBParser::parse(data, config)?;

    let mut config = HeadwayParseConfig::default();
    config.headers.customs_fee = "报关或其他费".into();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .load_worksheet(PATH_HEADWAY.clone(), SHEET_HEADWAY_2026)?
        .read()?;
    let shipment = HeadwayParser::parse(data, config)?;

    let processor = Processor::new([bill], [shipment])?;
    let (freight_bill, _freight_headway) = processor.get_freight()?;
    {
        // 判断一些必定存在的运单号
        let waybills: Vec<_> = freight_bill["运单号"]
            .str()?
            .into_iter()
            .flatten()
            .map(|s| s.to_string())
            .collect();
        let expected = [
            "WB2604098776",
            "WB2604097869",
            "WB2604168020",
            "WB2604165337",
        ];

        for e in expected {
            assert!(waybills.contains(&e.to_string()), "运费中缺少运单号: {}", e)
        }
    }

    let (customs_bill, _customs_headway) = processor.get_customs()?;
    println!("customs bill : {}", customs_bill);

    Ok(())
}
