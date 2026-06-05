use logirecon::{
    Result,
    parser::{
        BillValidated, HeadwayParser, Parse, ShipmentValidated, Validated, WBParser,
        user_input::UserInput,
    },
};

mod common;
use common::*;

#[test]
fn test_get_bill() -> Result<()> {
    let mut wb = WBParser::default();
    wb.provider_mut().add_sheets(PATH_BILLS, SHEET_WB);
    let df = wb.parse()?.get_valicated()?;
    println!("{}", df);
    assert!(df["日期"].is_not_null().all());
    Ok(())
}

#[test]
fn test_get_shipment() -> Result<()> {
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
    let bill: BillValidated = get_bill()?;
    let shipment: ShipmentValidated = get_shipment()?;
    let user_input = UserInput::new(bill, shipment)?;
    let (freight_bill, freight_headway) = user_input.get_freight()?;
    println!("freight bill : {}", freight_bill);
    println!("freight headway : {}", freight_headway);

    let (customs_bill, customs_headway) = user_input.get_customs()?;
    println!("customs bill : {}", customs_bill);
    println!("customs headway : {}", customs_headway);

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
