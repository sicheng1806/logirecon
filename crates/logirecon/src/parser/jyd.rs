use std::collections::HashMap;

use super::{AsHeaders, Parse};
use crate::validate::BillData;

/// 京奕达解析器
pub struct JydParser;

#[derive(Clone)]
/// 京奕达解析器配置
pub struct JydParseConfig {
    pub datefmt: String,
    pub year: i32,
    pub headers: JydHeaders,
}

impl Default for JydParseConfig {
    fn default() -> Self {
        use chrono::Datelike;
        Self {
            datefmt: "%-m月%-d".into(),
            year: chrono::Local::now().year(),
            headers: JydHeaders::default(),
        }
    }
}

crate::define_headers! {
    #[derive(Clone)]
    /// 京奕达表头
    pub struct JydHeaders [
        /// 日期
        date: "签入日期",
        /// 运单号
        waybill_no: "运单号",
        /// 客户运单号
        shipment_no: "FBA单号",
        /// 地址编码
        warehouse_code: "目的仓",
        /// 件数
        n_pieces: "件数",
        /// 收费重
        chargeable_weight: "收费重",
        /// 单价
        unit_price: "运费",
        /// 其他费用
        customs_fee: "其他费用"
    ]
}

impl Parse for JydParser {
    type Config = JydParseConfig;
    type Output = BillData;
    type Error = polars::error::PolarsError;

    fn parse(
        data: polars::prelude::DataFrame,
        config: Self::Config,
    ) -> Result<Self::Output, Self::Error> {
        use polars::prelude::*;
        let JydParseConfig {
            datefmt,
            year,
            headers,
        } = config;
        let forwarder = "京奕达";
        let name_mapping: HashMap<String, String> = headers.as_headers();
        let datefmt = format!("%Y-{datefmt}");

        // 日期
        let date = concat_str([lit(year.to_string()), col("签入日期")], "-", true)
            .str()
            .to_date(StrptimeOptions {
                format: Some(datefmt.into()),
                strict: false,
                exact: false,
                cache: true,
            })
            .alias("日期");
        // 运单号
        let waybill_no = col("运单号").str().strip_chars(lit(" ")).alias("运单号");
        // 货件单号
        let order_no = col("FBA单号")
            .str()
            .strip_chars(lit(" "))
            .str()
            .replace_all(lit(" "), lit(","), true)
            .str()
            .replace_all(lit("，"), lit(","), true)
            .alias("货件单号");
        // 物流中心编码
        let warehouse_code = col("目的仓")
            .str()
            .strip_chars(lit(" "))
            .alias("物流中心编码");
        // 货代名称
        let forwarder = lit(forwarder).alias("货代名称");
        // 单价
        let unit_price = col("运费").alias("运费单价");
        // 件数
        let n_pieces = col("件数").alias("件数");
        // 计费重
        let weight = col("收费重").alias("计费重");
        // 报关费
        let customs_fee = col("其他费用").alias("报关费");
        // 根据报关费和单价结合生成费用类型，报关费 -> 单价
        // {
        //     println!(
        //         "parse dataframe: \n{}",
        //         data.clone()
        //             .lazy()
        //             .select([concat_str(
        //                 [lit(year.to_string()), col("签入日期")],
        //                 "-",
        //                 true
        //             )])
        //             .collect()?
        //     );
        // }
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
                unit_price,
                customs_fee,
            ]);
        let df1 = df.clone().filter(col("运费单价").is_not_null()).select([
            all().exclude_cols(["运费单价", "报关费"]).as_expr(),
            col("运费单价").alias("单价"),
            lit("运费").alias("账单类型"),
        ]);
        let df2 = df.filter(col("报关费").is_not_null()).select([
            all().exclude_cols(["运费单价", "报关费"]).as_expr(),
            col("报关费").alias("单价"),
            lit("报关费").alias("账单类型"),
        ]);
        let df = concat([df1, df2], UnionArgs::default())?.collect()?;
        Ok(BillData(df))
    }
}
