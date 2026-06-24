use std::collections::HashMap;

use super::Parse;
use crate::{parser::AsHeaders, validate::ShipmentData};

/// 头程解析器
pub struct HeadwayParser;

#[derive(Clone)]
/// 头程解析器配置
pub struct HeadwayParseConfig {
    pub year: i32,
    pub headers: HeadwayHeaders,
}

crate::define_headers! {
    /// 头程表头
    #[derive(Clone)]
    pub struct HeadwayHeaders [
        /// 提货时间
        date: "提货时间",
        /// 货件单号
        shipment_no: "货件单号",
        /// 物流中心编码
        warehouse_code: "物流中心编码",
        /// 箱数
        n_pieces: "箱数",
        /// 货件计费重
        chargeable_weight: "货件计费重",
        /// 物流单价
        unit_price: "物流单价",
        /// 报关费
        customs_fee: "报关费",
        /// 报关周次
        customs_no: "报关周次",
    ]

}

impl Default for HeadwayParseConfig {
    fn default() -> Self {
        use chrono::Datelike;
        Self {
            year: chrono::Local::now().year(),
            headers: HeadwayHeaders::default(),
        }
    }
}

impl Parse for HeadwayParser {
    type Config = HeadwayParseConfig;
    type Output = ShipmentData;
    type Error = polars::error::PolarsError;

    fn parse(
        data: polars::prelude::DataFrame,
        config: Self::Config,
    ) -> Result<Self::Output, Self::Error> {
        use polars::prelude::*;
        let HeadwayParseConfig { year, headers } = config;
        let year_prefix = format!("Y{}", year);
        let name_mapping: HashMap<String, String> = headers.as_headers();

        //报关周次 添加年份
        let customs_no = concat_str(
            [
                lit(year_prefix.as_str()),
                when(col("报关周次").str().starts_with(lit("W")))
                    .then(col("报关周次").str().strip_chars(lit(" ")))
                    .otherwise(lit(NULL)),
            ],
            "",
            false,
        )
        .alias("报关周次");

        // 货件单号
        let shipment_no = col("货件单号")
            .str()
            .strip_chars(lit(" "))
            .alias("货件单号");

        // 物流中心编码
        let warehouse_code = col("物流中心编码").alias("物流中心编码");

        // 箱数
        let n_pieces = col("箱数").alias("件数");

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

        let df = data
            .lazy()
            .rename(name_mapping.values(), name_mapping.keys(), true)
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
            .collect()?;
        Ok(ShipmentData(df))
    }
}

#[cfg(test)]
mod tests {

    use crate::validate::IntoValidated;

    use super::*;
    use polars::prelude::*;

    #[test]
    fn test_parse() {
        let data: DataFrame = df!(
            "报关周次" => ["W15002"],
            "货件单号" => ["FBA15LMGRQ7F"],
            "物流中心编码" => ["BER8"],
            "箱数" => [144],
            "货件计费重" => [178.00],
            "物流单价" => [7.4],
            "报关或其他费" => [250.00],
            "提货时间" => ["2026/4/10"]
        )
        .unwrap();
        let mut config = HeadwayParseConfig::default();
        config.headers.customs_fee = "报关或其他费".to_string();
        let output = HeadwayParser::parse(data, config).unwrap();
        println!("{}", &output.0);
        let data = output.into_validated().unwrap();
        println!("{}", data);
    }
}
