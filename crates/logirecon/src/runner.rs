//! 为GUI界面提供统一的接口层和运行函数
//! # Example
//! ```ignore
//! fn main() {
//!     let wb = Template {
//!         parse_config: ParseConfig::WB(WBParseConfig::default()),
//!         sources: vec![ReadConfig::ExcelFilePath {
//!             path: PATH_BILLS.clone(),
//!             name: SHEET_WB.into(),
//!         }],
//!     };
//!     let grt = Template {
//!         parse_config: ParseConfig::GRT(WBParseConfig::grt()),
//!         sources: vec![ReadConfig::ExcelFilePath {
//!             path: PATH_BILLS.clone(),
//!             name: SHEET_GRT.into(),
//!         }],
//!    };
//!     let tsbg = Template {
//!         parse_config: ParseConfig::TS(TSParseConfig::default()),
//!         sources: vec![ReadConfig::ExcelFilePath {
//!             path: PATH_BILLS.clone(),
//!             name: SHEET_TSBG.into(),
//!         }],
//!     };
//!     let tsyf = Template {
//!         parse_config: ParseConfig::TS(TSParseConfig::default()),
//!         sources: vec![ReadConfig::ExcelFilePath {
//!             path: PATH_BILLS.clone(),
//!             name: SHEET_TSYF.into(),
//!         }],
//!     };
//!     let ddd = Template {
//!         parse_config: ParseConfig::DDD(DDDParseConfig::default()),
//!         sources: vec![ReadConfig::ExcelFilePath {
//!             path: PATH_BILLS.clone(),
//!             name: SHEET_DDD.into(),
//!         }],
//!     };
//!     let headway = {
//!         let mut config = HeadwayParseConfig::default();
//!         config.year = 2026;
//!         config.headers.customs_fee = "报关或其他费".into();
//!         Template {
//!             parse_config: ParseConfig::Headway(config),
//!             sources: vec![ReadConfig::ExcelFilePath {
//!                 path: PATH_HEADWAY.clone(),
//!                 name: SHEET_HEADWAY_2026.into(),
//!             }],
//!         }
//!     };
//!     let templates = vec![wb, grt, ddd, tsbg, tsyf, headway];
//!     let (freight_reconciler, customs_reconciler) = get_reconciler(templates).unwrap();
//!     let freight = freight_reconciler.get_long_result().unwrap();
//!     let customs = customs_reconciler.get_long_result().unwrap();
//!     let stasis_result = stasis_freight_and_customs(freight, customs).unwrap();
//!     println!("对账分析结果:\n{}", stasis_result);
//! }
use std::{
    io::{Read, Seek},
    path::PathBuf,
};

use crate::{
    parser::AsHeaders,
    process::ProcessError,
    reader::{ExcelError, ExcelReader},
    reconcile::{ReconcileError, Reconciler},
};

use super::DataFrame;
use super::parser::{DDDParseConfig, HeadwayParseConfig, TSParseConfig, WBParseConfig};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunError {
    #[error("读取文件失败, {0}")]
    Load(#[from] ExcelError),
    #[error("解析数据失败, {0}")]
    Parse(polars::error::PolarsError),
    #[error("数据转换失败, {0}")]
    Process(#[from] ProcessError),
    #[error("对账失败, {0}")]
    Reconsile(#[from] ReconcileError),
    #[error("数据处理失败, {0}")]
    Polars(#[from] polars::error::PolarsError),
    /// 用于GUI报错
    #[error("{0}")]
    Any(String),
}

/// 将模板序列转换为对账器
pub fn get_reconciler(
    templates: impl IntoIterator<Item = Template>,
) -> Result<(Reconciler, Reconciler), RunError> {
    use crate::prelude::*;

    // parse
    let mut bills: Vec<BillData> = vec![];
    let mut shipments: Vec<ShipmentData> = vec![];

    for template in templates.into_iter() {
        let Template {
            parse_config,
            sources,
        } = template;
        if sources.is_empty() {
            continue;
        }
        let headers = parse_config.as_headers();
        for read_config in sources {
            let data = match read_config {
                ReadConfig::ExcelFilePath { path, name } => ExcelReader::new(headers.values())
                    .primary("序号")
                    .load_worksheet(path, &name)?
                    .read()?,
                ReadConfig::ExcelReadSeek {
                    rs,
                    name,
                    extension,
                } => ExcelReader::new(headers.values())
                    .primary("序号")
                    .load_worksheet_from_rs(rs, &name, &extension)?
                    .read()?,
            };
            match &parse_config {
                ParseConfig::WB(config) => {
                    bills.push(WBParser::parse(data, config.to_owned()).map_err(RunError::Parse)?)
                }
                ParseConfig::TS(config) => {
                    bills.push(TSParser::parse(data, config.to_owned()).map_err(RunError::Parse)?)
                }
                ParseConfig::DDD(config) => {
                    bills.push(DDDParser::parse(data, config.to_owned()).map_err(RunError::Parse)?)
                }
                ParseConfig::Headway(config) => shipments
                    .push(HeadwayParser::parse(data, config.to_owned()).map_err(RunError::Parse)?),
            }
        }
    }
    // processor
    let processor = Processor::new(bills, shipments)?;
    let (freight_bill, freight_headway) = processor.get_freight()?;
    let (customs_bill, customs_headway) = processor.get_customs()?;
    // reconcile
    let freight_reconciler = ReconcileOption::freight()
        .left(freight_bill, "物流")
        .right(freight_headway, "我方")
        .try_into_reconciler()?
        .reconcile()?;
    let customs_reconciler = ReconcileOption::customs()
        .left(customs_bill, "物流")
        .right(customs_headway, "我方")
        .try_into_reconciler()?
        .reconcile()?;
    Ok((freight_reconciler, customs_reconciler))
}

/// 统计对账结果
///
/// # 参数
/// - freight: 运费对账的长格式结果
/// - customs: 报关费对账的长格式结果
///
/// 更多参见: [Reconciler]
pub fn stasis_freight_and_customs(
    freight: DataFrame,
    customs: DataFrame,
) -> Result<DataFrame, RunError> {
    use polars::prelude::*;
    let customs = customs
        .lazy()
        .select([
            col("运单号").str().split(lit(",")),
            col("_source").alias("数据来源"),
            col("金额").alias("报关费"),
            col("_summary").alias("报关费差异"),
        ])
        .explode(
            cols(["运单号"]),
            ExplodeOptions {
                empty_as_null: true,
                keep_nulls: false,
            },
        );
    let freight = freight.lazy().select([
        (col("单价") * col("计费重")).alias("预估运费"),
        col("运单号"),
        col("_source").alias("数据来源"),
        col("日期").alias("提货时间"),
        col("货代名称"),
        col("货件单号"),
        col("物流中心编码"),
        col("单价").alias("物流单价"),
        col("件数").alias("箱数"),
        col("计费重").alias("货件计费重"),
        col("_summary").alias("运费差异"),
    ]);
    let df = freight
        .join(
            customs,
            [col("运单号"), col("数据来源")],
            [col("运单号"), col("数据来源")],
            JoinArgs::new(JoinType::Full),
        )
        .filter(
            col("运费差异")
                .is_not_null()
                .or(col("报关费差异").is_not_null()),
        )
        .select([
            // 排序
            col("货代名称"),
            col("运单号"),
            col("数据来源"),
            col("提货时间"),
            col("货件单号"),
            col("物流中心编码"),
            col("物流单价"),
            col("箱数"),
            col("货件计费重"),
            col("预估运费"),
            col("报关费"),
            col("运费差异"),
            col("报关费差异"),
        ])
        .collect()?;
    Ok(df)
}

/// 数据源解析模板
///
/// 一个模板对应一个解析配置，可配置多个数据源数据
pub struct Template {
    pub parse_config: ParseConfig,
    pub sources: Vec<ReadConfig>,
}

/// 解析器配置
pub enum ParseConfig {
    WB(WBParseConfig),
    TS(TSParseConfig),
    DDD(DDDParseConfig),
    Headway(HeadwayParseConfig),
}

impl AsHeaders for ParseConfig {
    fn as_headers(&self) -> std::collections::HashMap<String, String> {
        match self {
            Self::WB(config) => config.headers.as_headers(),
            Self::TS(config) => config.headers.as_headers(),
            Self::DDD(config) => config.headers.as_headers(),
            Self::Headway(config) => config.headers.as_headers(),
        }
    }
    fn update_headers(&mut self, headers: impl IntoIterator<Item = (String, String)>) {
        match self {
            Self::WB(config) => config.headers.update_headers(headers),
            Self::TS(config) => config.headers.update_headers(headers),
            Self::DDD(config) => config.headers.update_headers(headers),
            Self::Headway(config) => config.headers.update_headers(headers),
        }
    }
}

/// 数据源的读取配置
pub enum ReadConfig {
    ExcelFilePath {
        path: PathBuf,
        name: String,
    },
    ExcelReadSeek {
        rs: Box<dyn ReadSeek>,
        name: String,
        extension: String,
    },
}

/// 用于 [`Read`] + [`Seek`] Trait 的动态特征对象
pub trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}
