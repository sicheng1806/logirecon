pub mod bill;
pub mod error;
pub mod excel;
pub mod parser;
pub mod reconsile;
pub mod relationship;
pub mod schema;
pub mod shipment;

pub use error::{Error, Result};
pub use excel::{ExcelReadOptions, ExcelReader};
pub use polars::datatypes::DataType;
pub use polars::prelude::{DataFrame, LazyFrame};
pub use schema::{AggOptions, Schema, Standardlize};

use std::sync::LazyLock;

pub static FREIGHT_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::default()
        .with_columns([
            ("运单号", (DataType::String, AggOptions::PK)),
            ("货件单号", (DataType::String, AggOptions::ByFirst)),
            ("日期", (DataType::Date, AggOptions::ByFirst)),
            ("物流中心编码", (DataType::String, AggOptions::ByFirst)),
            ("货代名称", (DataType::String, AggOptions::ByFirst)),
            ("件数", (DataType::Float64, AggOptions::ByFirst)),
            ("单价", (DataType::Float64, AggOptions::ByFirst)),
            ("计费重", (DataType::Float64, AggOptions::ByFirst)),
        ])
        .ok()
        .unwrap()
});

pub static CUSTOMS_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::default()
        .with_columns([
            ("报关周次", (DataType::String, AggOptions::PK)),
            ("货件单号", (DataType::String, AggOptions::ByFirst)),
            ("货代名称", (DataType::String, AggOptions::ByFirst)),
            ("金额", (DataType::Float64, AggOptions::ByFirst)),
        ])
        .ok()
        .unwrap()
});

pub static BILL_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    use polars::prelude::FrozenCategories;
    let fcats = FrozenCategories::new(["报关费", "运费"]).unwrap();
    Schema::default()
        .with_columns([
            ("运单号", (DataType::String, AggOptions::PK)),
            (
                "账单类型",
                (DataType::from_frozen_categories(fcats), AggOptions::PK),
                // (DataType::String, AggOptions::PK)
            ),
            //
            ("货件单号", (DataType::String, AggOptions::ByFirst)),
            // ("报关周次", (DataType::String, AggOptions::ByFirst)),
            ("日期", (DataType::Date, AggOptions::ByFirst)),
            ("物流中心编码", (DataType::String, AggOptions::ByFirst)),
            ("件数", (DataType::Float64, AggOptions::ByFirst)),
            ("货代名称", (DataType::String, AggOptions::ByFirst)),
            //
            ("单价", (DataType::Float64, AggOptions::BySum)),
            ("计费重", (DataType::Float64, AggOptions::ByFirst)),
        ])
        .ok()
        .unwrap()
});

pub static SHIPMENT_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::default()
        .with_columns([
            ("货件单号", (DataType::String, AggOptions::PK)),
            ("报关周次", (DataType::String, AggOptions::ByFirst)),
            ("日期", (DataType::Date, AggOptions::ByFirst)),
            ("物流中心编码", (DataType::String, AggOptions::ByFirst)),
            ("箱数", (DataType::Float64, AggOptions::BySum)),
            //
            ("计费重", (DataType::Float64, AggOptions::BySum)),
            ("单价", (DataType::Float64, AggOptions::BySum)),
            ("报关费", (DataType::Float64, AggOptions::ByFirst)),
        ])
        .ok()
        .unwrap()
});

pub static RELATIONSHIP_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::default()
        .with_columns([
            ("货运单号", (DataType::String, AggOptions::PK)),
            ("运单号", (DataType::String, AggOptions::ByFirst)),
            ("报关周次", (DataType::String, AggOptions::ByFirst)),
        ])
        .ok()
        .unwrap()
});
