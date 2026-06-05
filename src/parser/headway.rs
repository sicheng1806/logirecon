use chrono::{Datelike, Local};

use super::{Parse, SheetProvider, ShipmentValidated};
use crate::{LazyFrame, Result};

/// 头程数据解析器
pub struct HeadwayParser {
    pub provider: SheetProvider,
    pub year: i32,
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
            year: Local::now().year(),
        }
    }
}

impl Parse<ShipmentValidated> for HeadwayParser {
    fn provider(&self) -> &SheetProvider {
        &self.provider
    }

    fn provider_mut(&mut self) -> &mut SheetProvider {
        &mut self.provider
    }

    fn parse_dataframe(&self, dataframe: polars::prelude::DataFrame) -> Result<LazyFrame> {
        use polars::prelude::*;

        // let datefmt = &self.datefmt;
        let year_prefix = format!("Y{}", self.year);
        let name_mapping = self.provider.headers();
        let new: Vec<_> = name_mapping.keys().collect();
        let existing: Vec<_> = new.iter().map(|&k| name_mapping.get(k).unwrap()).collect();

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

        let df = dataframe.lazy().rename(existing, new, true).select([
            customs_no,
            shipment_no,
            warehouse_code,
            n_pieces,
            weight,
            unit_price,
            customs_fee,
            date,
        ]);

        Ok(df)
    }
}
