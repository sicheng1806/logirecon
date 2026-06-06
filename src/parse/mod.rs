pub mod headway;
pub mod parse;
pub mod provider;
pub mod user_input;
pub mod validate;
pub mod wb;

use super::DataType;
use std::sync::LazyLock;

// reuse
pub use headway::HeadwayParser;
pub use parse::Parse;
pub use provider::SheetProvider;
pub use validate::{BillValidated, DataSchema, ShipmentValidated, Validated};
pub use wb::WBParser;

#[derive(Debug, Clone, PartialEq)]
pub enum AggOption {
    PK,
    ByFirst,
    BySum,
}

// Schemas
static BILL_SCHEMA: LazyLock<DataSchema> = LazyLock::new(|| {
    use polars::prelude::FrozenCategories;
    let fcats = FrozenCategories::new(["运费", "报关费"]).unwrap();
    let iter = [
        ("运单号", (DataType::String, AggOption::PK)),
        (
            "账单类型",
            (DataType::from_frozen_categories(fcats), AggOption::PK),
            // (DataType::String, AggOptions::PK)
        ),
        //
        ("货件单号", (DataType::String, AggOption::ByFirst)),
        // ("报关周次", (DataType::String, AggOptions::ByFirst)),
        ("日期", (DataType::Date, AggOption::ByFirst)),
        ("物流中心编码", (DataType::String, AggOption::ByFirst)),
        ("件数", (DataType::Float64, AggOption::ByFirst)),
        ("货代名称", (DataType::String, AggOption::ByFirst)),
        //
        ("单价", (DataType::Float64, AggOption::BySum)),
        ("计费重", (DataType::Float64, AggOption::ByFirst)),
    ];
    DataSchema::from_iter(iter.map(|(n, s)| (n.to_string(), s)))
});

static SHIPMENT_SCHEMA: LazyLock<DataSchema> = LazyLock::new(|| {
    let iter = [
        ("货件单号", (DataType::String, AggOption::PK)),
        //
        ("报关周次", (DataType::String, AggOption::ByFirst)),
        ("日期", (DataType::Date, AggOption::ByFirst)),
        ("物流中心编码", (DataType::String, AggOption::ByFirst)),
        //
        ("件数", (DataType::Float64, AggOption::BySum)),
        ("计费重", (DataType::Float64, AggOption::BySum)),
        ("单价", (DataType::Float64, AggOption::ByFirst)),
        ("报关费", (DataType::Float64, AggOption::ByFirst)),
    ];
    DataSchema::from_iter(iter.map(|(n, s)| (n.to_string(), s)))
});

static FREIGHT_SCHEMA: LazyLock<DataSchema> = LazyLock::new(|| {
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
    DataSchema::from_iter(iter.map(|(n, s)| (n.to_string(), s)))
});

static CUSTOMS_SCHEMA: LazyLock<DataSchema> = LazyLock::new(|| {
    let iter = [
        ("报关周次", (DataType::String, AggOption::PK)),
        ("运单号", (DataType::String, AggOption::ByFirst)),
        ("货代名称", (DataType::String, AggOption::ByFirst)),
        ("金额", (DataType::Float64, AggOption::ByFirst)),
    ];

    DataSchema::from_iter(iter.map(|(n, s)| (n.to_string(), s)))
});
