use std::{
    collections::{HashMap, HashSet},
    ops::Sub,
};

use super::{ReconcileColumn, ReconcileError, Result};
use crate::DataFrame;

/// 对账器
///
/// 本质是DataFrame比对器, 通过设定各字段的对比方案  [ReconcileColumn] 来完成比对。
pub struct Reconciler {
    columns: HashMap<String, ReconcileColumn>,
    left: (String, DataFrame),
    right: (String, DataFrame),
    result: Option<DataFrame>,
}

impl Reconciler {
    /// 生成一个对比器
    pub fn new(
        columns: HashMap<String, ReconcileColumn>,
        left: (String, DataFrame),
        right: (String, DataFrame),
    ) -> Result<Self> {
        let pk: Vec<_> = columns
            .iter()
            .filter(|(_, column)| **column == ReconcileColumn::PK)
            .map(|(name, _)| name.to_string())
            .collect();
        if pk.len() != 1 {
            return Err(ReconcileError::PK(pk.len()));
        }
        // 验证方案
        let left_names: HashSet<_> = left
            .1
            .schema()
            .iter_names()
            .map(|name| name.to_string())
            .collect();
        let right_names: HashSet<_> = right
            .1
            .schema()
            .iter_names()
            .map(|name| name.to_string())
            .collect();
        let names: HashSet<_> = columns.keys().map(|t| t.to_string()).collect();
        if left_names != names {
            return Err(ReconcileError::NotMatch("左数据框不匹配列名".into()));
        }
        if right_names != names {
            return Err(ReconcileError::NotMatch("右数据框不匹配列名".into()));
        }
        Ok(Self {
            columns,
            left,
            right,
            result: None,
        })
    }

    /// 生成宽格式对比结果
    pub fn reconcile(mut self) -> Result<Self> {
        use polars::prelude::*;
        //
        let (left_name, left) = &self.left;
        let (right_name, right) = &self.right;
        let left_name = left_name.as_str();
        let right_name = right_name.as_str();
        let pk = self.primary_key().as_str();
        let name_suf = |suffix: &str, name: &str| format!("{}_{}", name, suffix);
        let select_expr = |suffix: &str| -> Vec<_> {
            let mut fields: Vec<_> = vec![col(pk).alias(name_suf(suffix, pk))];
            fields.extend(
                self.iter_reconciled()
                    .map(|(name, _)| col(name).alias(name_suf(suffix, name))),
            );
            fields
        };

        // 根据主键外连接，选择所有需匹配的字段添加相应后缀: A_left_name, A_right_name
        let left = left.clone().lazy().select(select_expr(left_name));
        let right = right.clone().lazy().select(select_expr(right_name));

        let both_expr = when(col(name_suf(left_name, pk)).is_null())
            .then(lit(right_name))
            .otherwise(
                when(col(name_suf(right_name, pk)).is_null())
                    .then(lit(left_name))
                    .otherwise(lit("both")),
            )
            .alias("_both");
        let pk_expr = when(col(name_suf(left_name, pk)).is_not_null())
            .then(col(name_suf(left_name, pk)))
            .otherwise(col(name_suf(right_name, pk)))
            .alias(pk);

        let result = left
            .full_join(
                right,
                name_suf(left_name, pk).as_str(),
                name_suf(right_name, pk).as_str(),
            )
            .with_columns([both_expr, pk_expr]);

        // 根据匹配方案生成比对结果列: diff_A
        let diff_exprs: Vec<_> = self
            .iter_reconciled()
            .map(|(name, column)| {
                let left_field = &name_suf(left_name, name);
                let right_field = &name_suf(right_name, name);
                let diff_field = name_suf(name, "diff");

                let expr = match column {
                    ReconcileColumn::Exact => when(col(left_field).eq(col(right_field)))
                        .then(lit(NULL))
                        .otherwise(
                            format_str("{} != {}", [col(left_field), col(right_field)]).unwrap(),
                        ),
                    ReconcileColumn::Numeric(atol) => when(
                        col(left_field)
                            .sub(col(right_field))
                            .abs()
                            .lt_eq(lit(*atol)),
                    )
                    .then(lit(NULL))
                    .otherwise(
                        format_str(
                            "|{} - {}| > {}",
                            [col(left_field), col(right_field), lit(*atol)],
                        )
                        .unwrap(),
                    ),
                    _ => lit(NULL),
                };
                expr.alias(diff_field)
            })
            .collect();
        let result = result.with_columns(diff_exprs);

        // 根据所有比对结果列生成汇总结果列: _summary

        let summary_expr = when(col("_both").eq(lit("both")))
            .then(concat_str(
                self.iter_reconciled()
                    .map(|(name, _)| {
                        format_str("{}:{}", [lit(name.as_str()), col(name_suf(name, "diff"))])
                            .unwrap()
                    })
                    .collect::<Vec<_>>(),
                ";",
                true,
            ))
            .otherwise(format_str("{} only", [col("_both")]).unwrap());
        let summary_expr = when(summary_expr.clone().eq(lit("")))
            .then(lit(NULL))
            .otherwise(summary_expr)
            .alias("_summary");

        let result = result.with_column(summary_expr);

        // 选择 过滤掉 PK_NAME 列
        let result = result
            .select([
                col(pk),
                all()
                    .exclude_cols([name_suf(left_name, pk), name_suf(right_name, pk), pk.into()])
                    .as_expr(),
            ])
            .collect()?;
        self.result = Some(result);
        Ok(self)
    }

    /// 生成宽格式对比结果
    pub fn get_width_result(&self) -> Result<DataFrame> {
        if let Some(result) = self.result.clone() {
            Ok(result)
        } else {
            Err(ReconcileError::NotReady(
                "需要先运行 build_result 方法".into(),
            ))
        }
    }

    /// 生成长格式对比结果
    pub fn get_long_result(&self) -> Result<DataFrame> {
        use polars::prelude::*;
        if let Some(result) = self.result.clone() {
            let (left_name, left) = &self.left;
            let (right_name, right) = &self.right;
            let pk = self.primary_key().as_str();

            let left_name = left_name.as_str();
            let right_name = right_name.as_str();
            let result = result.lazy().select([col(pk), col("_summary")]);

            let left = left
                .clone()
                .lazy()
                .with_column(lit(left_name).alias("_source"));

            let right = right
                .clone()
                .lazy()
                .with_column(lit(right_name).alias("_source"));
            let df =
                // concat 两表并添加 _source 字段
                concat([left, right], UnionArgs::default())?
                // 取宽结果的主键和_summary列 与之左连接
                .left_join(result, pk, pk)
                .select([col(pk), col("_source"), all().exclude_cols([pk, "_source", "_summary"]).as_expr(), col("_summary")])
                // 排序
                .sort([pk, "_source"], SortMultipleOptions::default());
            Ok(df.collect()?)
        } else {
            Err(ReconcileError::NotReady(
                "实现错误，需要先运行 build_result 方法".into(),
            ))
        }
    }

    fn primary_key(&self) -> &String {
        self.columns
            .iter()
            .find(|(_, column)| **column == ReconcileColumn::PK)
            .unwrap()
            .0
    }

    fn iter_reconciled(&self) -> impl Iterator<Item = (&String, &ReconcileColumn)> {
        self.columns.iter().filter(|(_, column)| {
            **column != ReconcileColumn::PK && **column != ReconcileColumn::None
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reconcile::ReconcileOption;

    #[test]
    fn test_resonsiler() -> Result<()> {
        use polars::prelude::*;

        let df1: DataFrame = df!("Score" => [99.0, 81.5, 75.],
                                 "No" => ["1", "2", "3"],
                                 "Name" => ["李勇", "张三", "李四"])?;

        let df2: DataFrame = df!("Score" => [99.5, 81.5, 75.],
                                 "No" => ["2", "1", "3"],
                                 "Name" => ["李勇", "张三", "胡五"])?;

        let reconciler = ReconcileOption::new_with_columns([
            ("Score", ReconcileColumn::Numeric(0.1)),
            ("Name", ReconcileColumn::PK),
            ("No", ReconcileColumn::Exact),
        ])
        .left(df1, "A")
        .right(df2, "B")
        .try_into_reconciler()?
        .reconcile()?;

        let width_res = reconciler.get_width_result()?;
        println!("width result : {}", width_res);
        let long_res = reconciler.get_long_result()?;
        println!("long result : {}", long_res);
        Ok(())
    }
}
