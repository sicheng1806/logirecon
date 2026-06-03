use chrono::{Datelike, Local};
use polars::prelude::DataFrame;

use super::{Parser, SheetProvider};
use crate::{Error, Result, pipeline::Shipment};

pub struct HeadwayParser {
    provider: SheetProvider,
    // datefmt: String,
    year: i32,
}

impl HeadwayParser {
    pub const DEFAULT_HEADERS: [&str; 8] = [
        "报关周次",
        "货件单号",
        "物流中心编码",
        "箱数",
        "货件计费重",
        "物流单价",
        "报关费",
        "提货时间",
    ];
}

impl Default for HeadwayParser {
    fn default() -> Self {
        Self {
            provider: SheetProvider::new(Self::DEFAULT_HEADERS, "序号"),
            // datefmt: "%Y/%-m/%d".into(),
            year: Local::now().year(),
        }
    }
}

impl Parser for HeadwayParser {
    fn provider(&self) -> &SheetProvider {
        &self.provider
    }

    fn provider_mut(&mut self) -> &mut SheetProvider {
        &mut self.provider
    }

    fn schema() -> crate::pipeline::Schema {
        Shipment::schema()
    }

    fn parse_dataframe(&self, dataframe: DataFrame) -> Result<DataFrame> {
        use polars::prelude::*;

        // let datefmt = &self.datefmt;
        let year = self.year;
        let name_mapping = self.provider.headers();
        let new: Vec<_> = name_mapping.keys().collect();
        let existing: Vec<_> = new.iter().map(|&k| name_mapping.get(k).unwrap()).collect();

        //报关周次 添加年份
        let customs_no = col("报关周次")
            .str()
            .strip_chars(lit(" "))
            .name()
            .prefix(format!("Y{}", year).as_str())
            .alias("报关周次");

        // 货件单号
        let shipment_no = col("货件单号")
            .str()
            .strip_chars(lit(" "))
            .alias("货件单号");

        // 物流中心编码
        let warehouse_code = col("物流中心编码").alias("物流中心编码");

        // 箱数
        let n_pieces = col("箱数").alias("箱数");

        // 计费重
        let weight = col("货件计费重").alias("计费重");

        // 单价
        let unit_price = col("物流单价").alias("单价");

        // 报关费
        let customs_fee = col("报关费").cast(DataType::Float64).alias("报关费");

        // 时间
        let date: Expr = col("提货时间")
            // .str()
            // .to_date(StrptimeOptions {
            //     format: Some(datefmt.into()),
            //     strict: false,
            //     ..Default::default()
            // })
            .alias("日期");

        let df = dataframe
            .lazy()
            .rename(existing, new, true)
            .select([
                customs_no,
                shipment_no,
                warehouse_code,
                n_pieces,
                weight,
                unit_price,
                customs_fee,
                date,
            ])
            .collect()
            .map_err(|e| Error::Process(format!("表格解析错误: {}", e)))?;

        Ok(df)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    #[test]
    fn test_parser() -> Result<()> {
        let mut parser = HeadwayParser::default();
        parser
            .provider_mut()
            .add_sheets(PATH_HEADWAY, SHEET_HEADWAY_2026)
            .update_headers([("报关费", "报关或其他费")]);
        // dbg!(parser.provider.headers());
        let df = parser.parse()?;
        println!("{}", df);
        Ok(())
    }
}
