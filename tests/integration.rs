mod common;
use common::*;
use logirecon::{
    BillValidated, HeadwayParser, Parse, Result, ShipmentValidated, Validated, WBParser,
};

#[test]
fn test_get_bill() -> Result<()> {
    use logirecon::{Parse, Validated, WBParser};
    let mut wb = WBParser::default();
    wb.provider_mut().add_sheets(PATH_BILLS, SHEET_WB);
    let df = wb.parse()?.get_valicated()?;
    println!("{}", df);
    assert!(df["日期"].is_not_null().all());
    Ok(())
}

#[test]
fn test_get_shipment() -> Result<()> {
    use logirecon::{HeadwayParser, Parse, Validated};
    let mut parser = HeadwayParser::default();
    parser
        .provider_mut()
        .add_sheets(PATH_HEADWAY, SHEET_HEADWAY_2026)
        .update_headers([("报关费", "报关或其他费")]);
    let df = parser.parse()?.get_valicated()?;
    println!("{}", df);
    assert!(df["报关周次"].has_nulls());
    Ok(())
}

#[test]
fn test_user_input() -> Result<()> {
    use logirecon::DataRepo;
    let bill = get_bill()?;
    {
        // 判断一些必定存在的运单号
        let waybills: Vec<String> = bill.get_valicated()?["运单号"]
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
            assert!(waybills.contains(&e.to_string()), "账单中缺少运单号: {}", e)
        }
    }

    let shipment = get_shipment()?;
    let user_input = DataRepo::new([bill], [shipment])?;
    let (freight_bill, _freight_headway) = user_input.get_freight()?;
    // println!("freight bill : {}", freight_bill);
    // println!("freight headway : {}", freight_headway);
    {
        // 判断一些必定存在的运单号
        let waybills: Vec<String> = freight_bill["运单号"]
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

    let (customs_bill, _customs_headway) = user_input.get_customs()?;
    println!("customs bill : {}", customs_bill);

    Ok(())
}

#[test]
fn test_reconsile() -> Result<()> {
    use logirecon::{
        DataRepo, ReconsileOption,
        reconsile::{CUSTOMS_RECONSILE_COLUMNS, FREIGHT_RECONSILE_COLUMNS},
    };
    use polars_excel_writer::PolarsExcelWriter;
    let bill = get_bill()?;
    let shipment = get_shipment()?;
    let user_input = DataRepo::new([bill], [shipment])?;
    let (freight_bill, freight_headway) = user_input.get_freight()?;
    let freight_reconsiler = ReconsileOption::new_with_columns(FREIGHT_RECONSILE_COLUMNS)
        .left(freight_bill, "物流")
        .right(freight_headway, "我方")
        .try_into_reconsiler()?
        .build_result()?;
    let freight_res = freight_reconsiler.get_long_result()?;
    println!("{}", freight_res);
    let (customs_bill, customs_headway) = user_input.get_customs()?;
    let customs_res = ReconsileOption::new_with_columns(CUSTOMS_RECONSILE_COLUMNS)
        .left(customs_bill, "物流")
        .right(customs_headway, "我方")
        .try_into_reconsiler()?
        .build_result()?
        .get_long_result()?;
    println!("{}", customs_res);
    let mut wb = PolarsExcelWriter::new();
    wb.set_worksheet_name("运费对账结果")?;
    wb.write_dataframe(&freight_res)?;
    wb.add_worksheet();
    wb.set_worksheet_name("报关费对账结果")?;
    wb.write_dataframe(&customs_res)?;
    wb.save("data/test/output.xlsx")?;
    Ok(())
}

fn get_shipment() -> Result<ShipmentValidated> {
    let mut parser = HeadwayParser::default();
    parser
        .provider_mut()
        .add_sheets(PATH_HEADWAY, SHEET_HEADWAY_2026)
        .update_headers([("报关费", "报关或其他费")]);
    let shipment = parser.parse()?;
    Ok(shipment)
}

fn get_bill() -> Result<BillValidated> {
    let mut parser = WBParser::default();
    parser.provider_mut().add_sheets(PATH_BILLS, SHEET_WB);
    let bill = parser.parse()?;
    Ok(bill)
}
