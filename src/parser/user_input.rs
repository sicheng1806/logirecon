use super::{BillValidated, CUSTOMS_SCHEMA, FREIGHT_SCHEMA, ShipmentValidated, Validated};
use crate::{DataFrame, Result};

/// 用户输入整合解析
///
/// 通过获取解析器验证后的数据 [BillValicated], [ShipmentValicated]，
/// 整合两者返回业务数据 Freight和 Customs
#[derive(Debug)]
pub struct UserInput {
    bill: DataFrame,
    shipment: DataFrame,
}

impl UserInput {
    pub fn new(bill: BillValidated, shipment: ShipmentValidated) -> Result<Self> {
        let bill = bill.get_valicated()?;
        let shipment = shipment.get_valicated()?;

        let relation: DataFrame = build_relation(&bill, &shipment)?;
        let bill: DataFrame = patch_bill(bill, &relation)?;
        let shipment: DataFrame = patch_shipment(shipment, &relation)?;
        Ok(Self { bill, shipment })
    }

    pub fn get_freight(&self) -> Result<(DataFrame, DataFrame)> {
        use polars::prelude::*;
        // 从"账单类型“ 为 "运费" 的条件中过滤出相关信息
        // println!("{}", self.bill.select(["账单类型", "运单号", "货件单号"])?);
        let freight_bill = self
            .bill
            .clone()
            .lazy()
            .filter(col("账单类型").eq(lit("运费")));
        // 按 运单号 分组 并 聚合
        let shipment_expr = col("货件单号").str().join(",", false).alias("货件单号");
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
            FREIGHT_SCHEMA.validate(freight_bill)?,
            FREIGHT_SCHEMA.validate(freight_shipment)?,
        ))
    }

    pub fn get_customs(&self) -> Result<(DataFrame, DataFrame)> {
        use polars::prelude::*;

        // 从"账单类型" 为 报关费 的条件中过滤出相关信息
        let customs_bill = self.bill.clone().lazy().group_by(["报关周次"]).agg([
            col("运单号").str().join(",", false).alias("运单号"),
            col("货代名称").first_non_null().alias("货代名称"),
            col("单价").first_non_null().alias("金额"),
        ]);
        let customs_shipment = self.shipment.clone().lazy().group_by(["报关周次"]).agg([
            col("货代名称").first_non_null().alias("货代名称"),
            col("报关费").first_non_null().alias("金额"),
            col("运单号").str().join(",", false).alias("运单号"),
        ]);
        Ok((
            CUSTOMS_SCHEMA.validate(customs_bill)?,
            CUSTOMS_SCHEMA.validate(customs_shipment)?,
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
    // 补充cusoms_no列
    let relation =
        relation
            .clone()
            .lazy()
            .select([col("报关周次"), col("运单号"), col("货代名称")]);
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
    // println!("relation is : {}", df);

    Ok(df)
}
