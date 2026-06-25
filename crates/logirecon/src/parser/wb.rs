use std::collections::HashMap;

use super::{AsHeaders, Parse};
use crate::validate::BillData;

/// 万邦解析器
pub struct WbParser;

#[derive(Clone)]
/// 万邦解析器配置
pub struct WBParseConfig {
    pub datefmt: String,
    pub units: (String, String),
    pub forwarder: String,
    pub headers: WbHeaders,
}

crate::define_headers! {
    #[derive(Clone)]
    /// 万邦表头
    pub struct WbHeaders [
        /// 日期
        date: "日期",
        /// 运单号
        waybill_no: "运单号",
        /// 订单号
        shipment_no: "订单号",
        /// 仓库编码
        warehouse_code: "仓库编码",
        /// 件数
        n_pieces: "件数",
        /// 收费重
        chargeable_weight: "收费重",
        /// 单价
        unit_price: "单价",
    ]
}

impl Default for WBParseConfig {
    fn default() -> Self {
        Self {
            datefmt: "%Y/%m/%d".into(),
            units: ("KG".to_string(), "票".to_string()),
            forwarder: "万邦".to_string(),
            headers: WbHeaders::default(),
        }
    }
}

impl WBParseConfig {
    /// 国润通模板的默认配置
    pub fn grt() -> Self {
        Self {
            forwarder: "国润通".into(),
            headers: WbHeaders {
                shipment_no: "扩展单号".into(),
                warehouse_code: "地址编码".into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    /// 积米默认模板配置
    pub fn jm() -> Self {
        Self {
            forwarder: "积米".to_string(),
            headers: WbHeaders {
                shipment_no: "客户运单号".into(),
                warehouse_code: "仓库代码".into(),
                chargeable_weight: "收费重(KG)".into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl Parse for WbParser {
    type Config = WBParseConfig;
    type Output = BillData;
    type Error = polars::error::PolarsError;

    fn parse(
        data: polars::prelude::DataFrame,
        config: Self::Config,
    ) -> Result<Self::Output, Self::Error> {
        use polars::prelude::*;
        let WBParseConfig {
            datefmt,
            units,
            forwarder,
            headers,
        } = config;
        let name_mapping: HashMap<String, String> = headers.as_headers();

        // 日期
        let date = col("日期")
            .str()
            .to_date(StrptimeOptions {
                format: Some(datefmt.into()),
                strict: false,
                exact: true,
                cache: true,
            })
            .alias("日期");
        // 运单号
        let waybill_no = col("运单号").str().strip_chars(lit(" ")).alias("运单号");
        // 货件单号
        let order_no = col("订单号")
            .str()
            .strip_chars(lit(" "))
            .str()
            .replace_all(lit(" "), lit(","), true)
            .str()
            .replace_all(lit("，"), lit(","), true)
            .alias("货件单号");
        // 物流中心编码
        let warehouse_code = col("仓库编码")
            .str()
            .strip_chars(lit(" "))
            .alias("物流中心编码");
        // 货代名称
        let forwarder = lit(forwarder).alias("货代名称");
        // 单价分列
        let split_unit_price = col("单价")
            .str()
            .splitn(lit("/"), 2)
            .struct_()
            .rename_fields(["单价", "账单类型"])
            .alias("to_split");
        // 账单类型
        let btype_col = when(col("账单类型").eq(lit(units.0.as_str())))
            .then(lit("运费"))
            .otherwise(lit("报关费"))
            .alias("账单类型");
        // 件数
        let n_pieces = col("件数").alias("件数");
        // 计费重
        let weight = col("收费重").alias("计费重");

        let df = data
            .lazy()
            .rename(name_mapping.values(), name_mapping.keys(), true)
            .select([
                date,
                waybill_no,
                order_no,
                warehouse_code,
                forwarder,
                n_pieces,
                weight,
                split_unit_price,
            ])
            .unnest(cols(["to_split"]), None)
            .with_column(btype_col)
            .collect()?;
        Ok(BillData(df))
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
            "发货日期" => ["2026/04/04",],
            "运单号" => ["WB2604024559"],
            "订单号" => ["FBA199JBH82C,FBA199JCMDW7,FBA199KNT8RY"],
            "仓库编码" => ["GEU2"],
            "件数" => [141],
            "收费重" => [1899],
            "单价" => ["3.60/KG"],
        )
        .unwrap();
        let mut config = WBParseConfig::default();
        config.headers.date = "发货日期".to_string();
        let bill = WbParser::parse(data, config).unwrap();
        println!("{}", &bill.0);
        let data = bill.into_validated().unwrap();
        println!("{}", data);
    }
}
