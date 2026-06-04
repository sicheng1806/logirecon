use logirecon::{
    Result,
    parser::{HeadwayParser, Parser, WBParser},
};
mod common;

#[test]
fn test_wb_parser() -> Result<()> {
    use common::{PATH_BILLS, SHEET_WB};
    let mut parser = WBParser::default();
    parser.provider_mut().add_sheets(PATH_BILLS, SHEET_WB);
    let df = parser.parse()?;
    println!("{}", df);
    Ok(())
}

#[test]
fn test_headway_parser() -> Result<()> {
    use common::{PATH_HEADWAY, SHEET_HEADWAY_2026};
    let mut parser = HeadwayParser::default();
    parser
        .provider_mut()
        .add_sheets(PATH_HEADWAY, SHEET_HEADWAY_2026)
        .update_headers([("报关费", "报关费或其他费")]);

    let df = parser.parse()?;
    println!("{}", df);
    Ok(())
}
