mod common;
use common::*;
use logirecon::{
    DataFrame,
    parser::*,
    process::Processor,
    reader::ExcelReader,
    reconcile::ReconcileOption,
    validate::{BillData, ShipmentData},
};
use polars::io::SerWriter;

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn stasis_freight_and_customs(freight: DataFrame, customs: DataFrame) -> Result<DataFrame> {
    use polars::prelude::*;
    let customs = customs
        .lazy()
        .select([
            col("运单号").str().split(lit(",")),
            col("_source").alias("数据来源"),
            col("金额").alias("报关费"),
            col("_summary").alias("报关费差异"),
        ])
        .explode(
            cols(["运单号"]),
            ExplodeOptions {
                empty_as_null: true,
                keep_nulls: false,
            },
        );
    let freight = freight.lazy().select([
        (col("单价") * col("计费重")).alias("预估运费"),
        col("运单号"),
        col("_source").alias("数据来源"),
        col("日期").alias("提货时间"),
        col("货代名称"),
        col("货件单号"),
        col("物流中心编码"),
        col("单价").alias("物流单价"),
        col("件数").alias("箱数"),
        col("计费重").alias("货件计费重"),
        col("_summary").alias("运费差异"),
    ]);
    let df = freight
        .join(
            customs,
            [col("运单号"), col("数据来源")],
            [col("运单号"), col("数据来源")],
            JoinArgs::new(JoinType::Full),
        )
        .filter(
            col("运费差异")
                .is_not_null()
                .or(col("报关费差异").is_not_null()),
        )
        .select([
            // 排序
            col("货代名称"),
            col("运单号"),
            col("数据来源"),
            col("提货时间"),
            col("货件单号"),
            col("物流中心编码"),
            col("物流单价"),
            col("箱数"),
            col("货件计费重"),
            col("预估运费"),
            col("报关费"),
            col("运费差异"),
            col("报关费差异"),
        ])
        .collect()?;
    Ok(df)
}

#[test]
fn test_all_process() -> Result<()> {
    let mut bills: Vec<BillData> = vec![];
    let mut shipments: Vec<ShipmentData> = vec![];
    // wb grt
    let mut config = WBParseConfig::default();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .primary("序号")
        .load_worksheet(PATH_BILLS.clone(), SHEET_WB)?
        .read()?;
    bills.push(WbParser::parse(data, config.clone())?);
    config.headers.shipment_no = "扩展单号".into();
    config.headers.warehouse_code = "地址编码".into();
    config.forwarder = "国润通".into();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .primary("序号")
        .load_worksheet(PATH_BILLS.clone(), SHEET_GRT)?
        .read()?;
    bills.push(WbParser::parse(data, config)?);
    // ts
    let config = TsParseConfig::default();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .primary("序号")
        .load_worksheet(PATH_BILLS.clone(), SHEET_TSBG)?
        .read()?;
    bills.push(TsParser::parse(data, config.clone())?);
    let data = ExcelReader::new(config.headers.as_headers().values())
        .primary("序号")
        .load_worksheet(PATH_BILLS.clone(), SHEET_TSYF)?
        .read()?;
    bills.push(TsParser::parse(data, config)?);
    // ddd
    let config = DddParseConfig::default();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .primary("序号")
        .load_worksheet(PATH_BILLS.clone(), SHEET_DDD)?
        .read()?;
    bills.push(DddParser::parse(data, config)?);
    // jyd
    let config = JydParseConfig::default();
    let data = ExcelReader::new(config.headers.as_headers().values())
        .primary("序号")
        .load_worksheet(PATH_BILLS.clone(), SHEET_JYD)?
        .read()?;
    bills.push(JydParser::parse(data, config)?);
    // headway
    let mut config = HeadwayParseConfig::default();
    let data_2025 = ExcelReader::new(config.headers.as_headers().values())
        .primary("序号")
        .load_worksheet(PATH_HEADWAY.clone(), SHEET_HEADWAY_2025)?
        .read()?;
    shipments.push(HeadwayParser::parse(data_2025, config.clone())?);
    config.headers.customs_fee = "报关或其他费".into();
    let data_2026 = ExcelReader::new(config.headers.as_headers().values())
        .primary("序号")
        .load_worksheet(PATH_HEADWAY.clone(), SHEET_HEADWAY_2026)?
        .read()?;
    shipments.push(HeadwayParser::parse(data_2026, config)?);
    println!("读取文件完毕");
    // processor
    let processor = Processor::new(bills, shipments)?;
    let (freight_bill, freight_headway) = processor.get_freight()?;
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

    let (customs_bill, customs_headway) = processor.get_customs()?;
    println!("customs bill : {}", customs_bill);

    // reconciler
    let freight_reconciler = ReconcileOption::freight()
        .left(freight_bill, "物流")
        .right(freight_headway, "我方")
        .try_into_reconciler()?
        .reconcile()?;
    println!("{}", freight_reconciler.get_long_result()?);
    let file = std::fs::File::create("tests/freight_output.csv")?;
    polars::prelude::CsvWriter::new(file).finish(&mut freight_reconciler.get_width_result()?)?;

    let customs_reconciler = ReconcileOption::customs()
        .left(customs_bill, "物流")
        .right(customs_headway, "我方")
        .try_into_reconciler()?
        .reconcile()?;

    let file = std::fs::File::create("tests/customs_output.csv")?;
    polars::prelude::CsvWriter::new(file).finish(&mut customs_reconciler.get_width_result()?)?;

    println!("{}", customs_reconciler.get_long_result()?);

    let mut stasis = stasis_freight_and_customs(
        freight_reconciler.get_long_result()?,
        customs_reconciler.get_long_result()?,
    )?;

    let file = std::fs::File::create("tests/stasis_output.csv")?;
    polars::prelude::CsvWriter::new(file).finish(&mut stasis)?;
    Ok(())
}
