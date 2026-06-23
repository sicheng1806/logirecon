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

/// 头程表头
#[derive(Clone)]
pub struct HeadwayHeaders {
    /// 提货时间
    pub date: String,
    /// 货件单号
    pub shipment_no: String,
    /// 物流中心编码
    pub warehouse_code: String,
    /// 箱数
    pub n_pieces: String,
    /// 货件计费重
    pub chargeable_weight: String,
    /// 物流单价
    pub unit_price: String,
    /// 报关费
    pub customs_fee: String,
    /// 报关周次
    pub customs_no: String,
}

impl Default for HeadwayHeaders {
    fn default() -> Self {
        Self {
            date: "提货时间".to_string(),
            shipment_no: "货件单号".to_string(),
            warehouse_code: "物流中心编码".to_string(),
            n_pieces: "箱数".to_string(),
            chargeable_weight: "货件计费重".to_string(),
            unit_price: "物流单价".to_string(),
            customs_fee: "报关费".to_string(),
            customs_no: "报关周次".to_string(),
        }
    }
}

impl AsHeaders for HeadwayHeaders {
    fn as_headers(&self) -> HashMap<String, String> {
        let HeadwayHeaders {
            date,
            shipment_no,
            warehouse_code,
            n_pieces,
            chargeable_weight,
            unit_price,
            customs_fee,
            customs_no,
        } = self;
        HashMap::from_iter(
            [
                ("提货时间", date),
                ("货件单号", shipment_no),
                ("物流中心编码", warehouse_code),
                ("箱数", n_pieces),
                ("货件计费重", chargeable_weight),
                ("物流单价", unit_price),
                ("报关费", customs_fee),
                ("报关周次", customs_no),
            ]
            .map(|(k, v)| (k.to_string(), v.to_owned())),
        )
    }
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
