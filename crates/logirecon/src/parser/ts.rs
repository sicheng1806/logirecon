use std::collections::HashMap;

use super::{AsHeaders, Parse};
use crate::validate::BillData;

/// 天盛解析器
pub struct TSParser;

#[derive(Clone)]
/// 天盛解析器配置
pub struct TSParseConfig {
    pub datefmt: String,
    pub units: (String, String),
    pub headers: TSHeaders,
}

impl Default for TSParseConfig {
    fn default() -> Self {
        Self {
            datefmt: "%Y-%m-%d".into(),
            units: (r#"(KG|立方|kg)"#.to_string(), r#"(票)"#.to_string()),
            headers: TSHeaders::default(),
        }
    }
}

crate::define_headers! {
    #[derive(Clone)]
    /// 天盛表头
    pub struct TSHeaders [
        /// 日期
        date: "日期",
        /// 运单号
        waybill_no: "运单号",
        /// 客户运单号
        shipment_no: "客户运单号",
        /// 地址编码
        warehouse_code: "地址编码",
        /// 件数
        n_pieces: "件数",
        /// 收费重
        chargeable_weight: "收费重",
        /// 单价
        unit_price: "单价",
        /// 单位
        unit: "单位",
    ]
}

impl Parse for TSParser {
    type Config = TSParseConfig;
    type Output = BillData;
    type Error = polars::error::PolarsError;

    fn parse(
        data: polars::prelude::DataFrame,
        config: Self::Config,
    ) -> Result<Self::Output, Self::Error> {
        use polars::prelude::*;
        let TSParseConfig {
            datefmt,
            units,
            headers,
        } = config;
        let forwarder = "天盛";
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
        let order_no = col("客户运单号")
            .str()
            .strip_chars(lit(" "))
            .str()
            .replace_all(lit(" "), lit(","), true)
            .str()
            .replace_all(lit("，"), lit(","), true)
            .alias("货件单号");
        // 物流中心编码
        let warehouse_code = col("地址编码")
            .str()
            .strip_chars(lit(" "))
            .alias("物流中心编码");
        // 货代名称
        let forwarder = lit(forwarder).alias("货代名称");
        // 单价
        let unit_price = col("单价").alias("单价");
        // 账单类型
        let bill_type = when(col("单位").str().contains(lit(units.1.as_str()), false))
            .then(lit("报关费"))
            .otherwise(lit("运费"))
            .alias("账单类型");
        // 件数
        let n_pieces = col("件数").alias("件数");
        // 计费重
        let weight = col("收费重").alias("计费重");

        // println!("parse dataframe: \n{}", dataframe);
        let df = data
            .lazy()
            .rename(name_mapping.values(), name_mapping.keys(), true)
            .select(name_mapping.keys().map(col).collect::<Vec<_>>())
            .select([
                date,
                waybill_no,
                order_no,
                warehouse_code,
                forwarder,
                n_pieces,
                weight,
                bill_type,
                unit_price,
            ])
            .collect()?;
        Ok(BillData(df))
    }
}
