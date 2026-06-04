use polars::prelude::DataFrame;

use super::{Parser, SheetProvider};
use crate::{BILL_SCHEMA, Result, Standardlize};

pub struct WBParser {
    provider: SheetProvider,
    datefmt: String,
    units: (String, String),
}

impl WBParser {
    pub const DEFAULT_HEADERS: [&str; 7] = [
        "日期",
        "运单号",
        "订单号",
        "仓库编码",
        "件数",
        "收费重",
        "单价",
    ];
}

impl Default for WBParser {
    fn default() -> Self {
        Self {
            provider: SheetProvider::new(Self::DEFAULT_HEADERS, "序号"),
            datefmt: "%Y/%m/%d".into(),
            units: ("KG".into(), "票".into()),
        }
    }
}

impl Standardlize for WBParser {
    fn standardlize(&self, df: polars::prelude::LazyFrame) -> Result<polars::prelude::LazyFrame> {
        BILL_SCHEMA.standardlize(df)
    }
}

impl Parser for WBParser {
    fn provider(&self) -> &SheetProvider {
        &self.provider
    }

    fn provider_mut(&mut self) -> &mut SheetProvider {
        &mut self.provider
    }

    fn parse_dataframe(&self, dataframe: DataFrame) -> Result<DataFrame> {
        use polars::prelude::*;
        let datefmt = &self.datefmt;
        let units = &self.units;
        let forwarder = "万邦";
        let name_mapping = self.provider.headers();

        // 日期
        let date = col("日期")
            .str()
            .to_date(StrptimeOptions {
                format: Some(datefmt.into()),
                strict: true,
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
        let btype_col = when(col("账单类型") == lit(units.0.as_str()))
            .then(lit("运费"))
            .otherwise(lit("报关费"))
            .alias("账单类型");
        // 件数
        let n_pieces = col("件数").alias("件数");
        // 计费重
        let weight = col("收费重").alias("计费重");

        let df = dataframe
            .lazy()
            .rename(name_mapping.values(), name_mapping.keys(), true)
            .select(&name_mapping.keys().map(|t| col(t)).collect::<Vec<_>>())
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
        // dbg!(&df);
        Ok(df)
    }
}