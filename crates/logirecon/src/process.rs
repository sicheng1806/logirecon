use crate::DataFrame;
use crate::validate::{
    AggOption, BillData, DataType, IntoValidated, SchemaValidator, ShipmentData,
};
use std::sync::LazyLock;
use thiserror::Error;

pub static FREIGHT_SCHEMA: LazyLock<SchemaValidator> = LazyLock::new(|| {
    let iter = [
        ("运单号", (DataType::String, AggOption::PK)),
        ("货件单号", (DataType::String, AggOption::ByFirst)),
        ("日期", (DataType::Date, AggOption::ByFirst)),
        ("物流中心编码", (DataType::String, AggOption::ByFirst)),
        ("货代名称", (DataType::String, AggOption::ByFirst)),
        //
        ("件数", (DataType::Float64, AggOption::ByFirst)),
        ("单价", (DataType::Float64, AggOption::ByFirst)),
        ("计费重", (DataType::Float64, AggOption::ByFirst)),
    ];
    SchemaValidator::from_iter(iter.map(|(n, s)| (n.to_string(), s)))
});

pub static CUSTOMS_SCHEMA: LazyLock<SchemaValidator> = LazyLock::new(|| {
    let iter = [
        ("报关周次", (DataType::String, AggOption::PK)),
        ("运单号", (DataType::String, AggOption::ByFirst)),
        ("货代名称", (DataType::String, AggOption::ByFirst)),
        ("金额", (DataType::Float64, AggOption::ByFirst)),
    ];

    SchemaValidator::from_iter(iter.map(|(n, s)| (n.to_string(), s)))
});

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("数据处理错误 {0}")]
    Polar(#[from] polars::error::PolarsError),
    #[error("数据为空无法处理: {0}")]
    Null(String),
}

type Result<T, E = ProcessError> = std::result::Result<T, E>;

/// 用户输入整合解析
///
/// 通过获取解析器验证后的数据 [BillData], [ShipmentData]，
/// 整合两者返回业务数据 Freight和 Customs
#[derive(Debug)]
pub struct Processor {
    bill: DataFrame,
    shipment: DataFrame,
}

impl Processor {
    pub fn new(
        bills: impl IntoIterator<Item = BillData>,
        shipments: impl IntoIterator<Item = ShipmentData>,
    ) -> Result<Self> {
        use polars::prelude::*;
        // 获取验证数据并合并
        let mut raw_bills = vec![];
        let mut raw_shipments = vec![];
        for bill in bills.into_iter() {
            raw_bills.push(bill.into_validated()?.lazy());
        }
        for shipment in shipments.into_iter() {
            raw_shipments.push(shipment.into_validated()?.lazy());
        }
        if raw_bills.is_empty() {
            return Err(ProcessError::Null("物流账单数据为空".to_string()));
        }
        if raw_shipments.is_empty() {
            return Err(ProcessError::Null("我方明细数据为空".into()));
        }

        let bill = concat(raw_bills, UnionArgs::default())?.collect()?;
        let shipment = concat(raw_shipments, UnionArgs::default())?.collect()?;

        // parse
        let relation: DataFrame = build_relation(&bill, &shipment)?;
        let bill: DataFrame = patch_bill(bill, &relation)?;
        let shipment: DataFrame = patch_shipment(shipment, &relation)?;
        Ok(Self { bill, shipment })
    }

    pub fn get_freight(&self) -> Result<(DataFrame, DataFrame)> {
        use polars::prelude::*;
        // bill: 从"账单类型“ 为 "运费" 的条件中过滤出相关信息
        let freight_bill = self
            .bill
            .clone()
            .lazy()
            .filter(col("账单类型").eq(lit("运费")));
        // shipment: 按 运单号 分组 并 聚合
        let shipment_expr = col("货件单号")
            .unique_stable()
            .str()
            .join(",", false)
            .alias("货件单号");
        let sum_exprs = ["件数", "计费重"].map(|t| col(t).sum().alias(t));
        let first_exprs =
            ["单价", "日期", "物流中心编码", "货代名称"].map(|t| col(t).first_non_null().alias(t));
        let mut agg_exprs: Vec<_> = vec![shipment_expr];
        agg_exprs.extend(sum_exprs);
        agg_exprs.extend(first_exprs);
        let freight_shipment = self
            .shipment
            .clone()
            .lazy()
            .group_by(["运单号"])
            .agg(&agg_exprs);
        Ok((
            FREIGHT_SCHEMA.validate(freight_bill.collect()?)?,
            FREIGHT_SCHEMA.validate(freight_shipment.collect()?)?,
        ))
    }

    pub fn get_customs(&self) -> Result<(DataFrame, DataFrame)> {
        use polars::prelude::*;

        // 从"账单类型" 为 报关费 的条件中过滤出相关信息
        let bill = self.bill.clone().lazy();
        let custom_no_with_shipment = bill
            .clone()
            .group_by(["报关周次"])
            .agg([col("运单号").unique_stable().str().join(",", false)]);
        let customs_bill = bill
            .filter(col("账单类型").eq(lit("报关费")))
            .group_by(["报关周次"])
            .agg([
                col("货代名称").first_non_null().alias("货代名称"),
                col("单价").first_non_null().alias("金额"),
            ])
            .left_join(custom_no_with_shipment, "报关周次", "报关周次");
        // {
        //     let mut file = std::fs::File::create("data/test/customs_bill.csv").unwrap();
        //     let mut df = customs_bill.clone().collect()?;
        //     CsvWriter::new(&mut file).finish(&mut df)?;
        // }
        let customs_shipment = self.shipment.clone().lazy().group_by(["报关周次"]).agg([
            col("货代名称").first_non_null().alias("货代名称"),
            col("报关费").first_non_null().alias("金额"),
            col("运单号")
                .unique_stable()
                .str()
                .join(",", false)
                .alias("运单号"),
        ]);
        Ok((
            CUSTOMS_SCHEMA.validate(customs_bill.collect()?)?,
            CUSTOMS_SCHEMA.validate(customs_shipment.collect()?)?,
        ))
    }
}

fn patch_shipment(shipment: DataFrame, relation: &DataFrame) -> Result<DataFrame> {
    use polars::prelude::*;
    // 补充waybill_no列
    let relation =
        relation
            .clone()
            .lazy()
            .select([col("货件单号"), col("运单号"), col("货代名称")]);
    let df = relation
        .left_join(shipment.lazy(), "货件单号", "货件单号")
        .collect()?;
    // println!("patched shipment is : {}", df);
    Ok(df)
}

fn patch_bill(bill: DataFrame, relation: &DataFrame) -> Result<DataFrame> {
    use polars::prelude::*;
    // println!("to patched bill is : {}", bill);
    let relation = relation
        .clone()
        .lazy()
        .select([col("报关周次"), col("运单号"), col("货代名称")])
        .group_by(["运单号"])
        .agg([
            col("报关周次").first_non_null(),
            col("货代名称").first_non_null(),
        ]);
    let df = bill
        .lazy()
        .left_join(relation, "运单号", "运单号")
        .collect()?;
    // println!("patched bill is : {}", df);

    Ok(df)
}

fn build_relation(bill: &DataFrame, shipment: &DataFrame) -> Result<DataFrame> {
    use polars::prelude::*;
    let bill = bill.clone().lazy();
    let shipment = shipment.clone().lazy();

    // 从 bill 从提取 shipment_no
    let split_expr = col("货件单号")
        .str()
        .strip_chars(lit(" "))
        .str()
        .replace_all(lit(" "), lit(","), true)
        .str()
        .replace_all(lit("，"), lit(","), true)
        .str()
        .split(lit(","))
        .alias("货件单号");
    let df = bill
        .select([col("运单号"), split_expr, col("货代名称")])
        .explode(
            cols(["货件单号"]),
            ExplodeOptions {
                empty_as_null: true,
                keep_nulls: true,
            },
        )
        // 确保shipment_no为主键
        .group_by(["货件单号"])
        .agg([
            col("运单号").first_non_null(),
            col("货代名称").first_non_null(),
        ])
        .inner_join(
            shipment.select([col("货件单号"), col("报关周次")]),
            "货件单号",
            "货件单号",
        )
        .collect()?;

    Ok(df)
}
