use std::collections::HashMap;

use super::{AsHeaders, Parse};
use crate::validate::BillData;

/// 嘀嗒嘀解析器
pub struct DDDParser;

#[derive(Clone)]
/// 嘀嗒嘀解析器配置
pub struct DDDParseConfig {
    pub datefmt: String,
    pub headers: DDDHeaders,
}

crate::define_headers! {
    #[derive(Debug, Clone)]
    /// 嘀嗒嘀表头
    pub struct DDDHeaders [
        /// 日期
        date: "签入日期",
        /// 运单号
        waybill_no: "运单号",
        /// 订单号
        shipment_no: "FBA单号",
        /// 仓库编码
        warehouse_code: "目的仓",
        /// 件数
        n_pieces: "件数",
        /// 收费重
        chargeable_weight: "收费重",
        /// 计算公式
        formula: "计算公式",
    ]
}

impl Default for DDDParseConfig {
    fn default() -> Self {
        Self {
            datefmt: "%Y-%m-%d".into(),
            headers: DDDHeaders::default(),
        }
    }
}

impl Parse for DDDParser {
    type Config = DDDParseConfig;
    type Output = BillData;
    type Error = polars::error::PolarsError;

    fn parse(
        data: polars::prelude::DataFrame,
        config: Self::Config,
    ) -> Result<Self::Output, Self::Error> {
        use polars::prelude::*;
        let DDDParseConfig { datefmt, headers } = config;
        let forwarder = "嘀嗒嘀";
        let name_mapping: HashMap<String, String> = headers.as_headers();

        let date = col("签入日期")
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

        // 件数
        let n_pieces = col("件数").alias("件数");
        // 计费重
        let weight = col("收费重").alias("计费重");
        // 运费单价
        let unit_price = col("计算公式")
            .str()
            .extract(lit(r#"<\d\.?\d*\*(\d+\.?\d*)"#), 1)
            .cast(DataType::Float64)
            .alias("运费单价");
        // 报关费
        let customs_fee1 = col("计算公式")
            .str()
            .extract_all(lit(r#"<(\d+\.?\d*)>"#))
            .list()
            .eval(col("").str().replace_all(lit("<|>"), lit(""), false))
            .list()
            .eval(col("").cast(DataType::Float64))
            .alias("报关费");
        let customs_filter = col("报关费").list().len().gt(lit(0));
        let customs_fee2 = col("报关费").list().sum().alias("单价");
        // concat 运费单价和报关费单价并添加来源列：账单类型

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
                unit_price,
                customs_fee1,
            ]);
        let df1 = df.clone().filter(col("运费单价").is_not_null()).select([
            all().exclude_cols(["运费单价", "报关费"]).as_expr(),
            col("运费单价").alias("单价"),
            lit("运费").alias("账单类型"),
        ]);
        let df2 = df.filter(customs_filter).select([
            all().exclude_cols(["运费单价", "报关费"]).as_expr(),
            customs_fee2,
            lit("报关费").alias("账单类型"),
        ]);
        let df = concat([df1, df2], UnionArgs::default())?.collect()?;

        // {
        //     let mut file = std::fs::File::create("data/test/output.csv").unwrap();
        //     CsvWriter::new(&mut file).finish(&mut df.clone().collect()?)?;
        // }

        Ok(BillData(df))
    }
}
