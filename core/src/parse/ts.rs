use super::{BillValidated, Parse, SheetProvider};
use crate::{LazyFrame, Result};

/// 天盛数据解析器
///
/// 默认表头: [TSParser::DEFAULT_HEADERS]
pub struct TSParser {
    pub provider: SheetProvider,
    pub datefmt: String,
    pub units: (String, String),
}

impl TSParser {
    pub const DEFAULT_HEADERS: [&str; 8] = [
        "日期",
        "运单号",
        "客户运单号",
        "地址编码",
        "件数",
        "收费重",
        "单价",
        "单位",
    ];
}

impl Default for TSParser {
    fn default() -> Self {
        Self {
            provider: SheetProvider::new(Self::DEFAULT_HEADERS, "序号"),
            datefmt: "%Y-%m-%d".into(),
            units: (r#"(KG|立方|kg)"#.to_string(), r#"(票)"#.to_string()),
        }
    }
}

impl Parse<BillValidated> for TSParser {
    fn provider(&self) -> &SheetProvider {
        &self.provider
    }

    fn provider_mut(&mut self) -> &mut SheetProvider {
        &mut self.provider
    }

    fn parse_dataframe(&self, dataframe: polars::prelude::DataFrame) -> Result<LazyFrame> {
        use polars::prelude::*;
        let datefmt = &self.datefmt;
        let units = &self.units;
        let forwarder = "天盛";
        let name_mapping = self.provider.headers();

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
        let df = dataframe
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
            ]);
        // dbg!(&df);
        Ok(df)
    }
}
