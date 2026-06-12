use super::{UserData, dataframe_table};
use crate::{CONTAINER_PADDING, EXCEL_SUFFIX, H1_SIZE, SPACING, modal};
use iced::{
    Element,
    Length::*,
    Task,
    widget::{button, column, container, row, scrollable, space, text},
};
use logirecon_core::{BillValidated, ShipmentValidated};
use polars::prelude::DataFrame;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum DumpType {
    Result,
    Statis,
}

type RunResult = Result<(DataFrame, DataFrame), logirecon_core::Error>;
type DumpResult = Result<String, logirecon_core::Error>;

#[derive(Debug, Clone)]
pub enum RunMessage {
    Run,
    UpdateResult(RunResult),
    ShowResult,
    HideResult,

    Dump(DumpType),
    Dumped(DumpResult),
    ShowDumpResult,
    HideDumpResult,
}

/// 用于展示用户输入的默认运行组件
#[derive(Debug, Clone, Default)]
pub struct Runner {
    pub show_result: bool,
    pub running: bool,
    pub result: Option<RunResult>,

    pub dump_result: Option<DumpResult>,
    pub show_dump_result: bool,
    pub dumpping: bool,
}

impl Runner {
    pub fn update(&mut self, message: RunMessage, details: [Vec<UserData>; 2]) -> Task<RunMessage> {
        match message {
            RunMessage::Run => {
                if !self.running {
                    self.result = None;
                    self.running = true;
                    Task::perform(Self::run(details), RunMessage::UpdateResult)
                } else {
                    Task::none()
                }
            }
            RunMessage::UpdateResult(res) => {
                self.result = Some(res);
                self.running = false;
                Task::done(RunMessage::ShowResult)
            }
            RunMessage::ShowResult => {
                self.show_result = true;
                Task::none()
            }
            RunMessage::HideResult => {
                self.show_result = false;
                Task::none()
            }

            RunMessage::Dump(dump_type) => {
                if let Some(res) = self.result.as_ref()
                    && !self.dumpping
                    && let Ok((freight, customs)) = res.as_ref()
                {
                    self.dumpping = true;
                    self.dump_result = None;
                    match dump_type {
                        DumpType::Result => Task::perform(
                            Self::dump_to_excel(freight.clone(), customs.clone()),
                            RunMessage::Dumped,
                        ),
                        DumpType::Statis => Task::perform(
                            Self::dump_statis_to_excel(freight.clone(), customs.clone()),
                            RunMessage::Dumped,
                        ),
                    }
                } else {
                    Task::none()
                }
            }

            RunMessage::Dumped(res) => {
                self.dumpping = false;
                self.dump_result = Some(res);
                self.show_dump_result = true;
                Task::done(RunMessage::ShowDumpResult)
            }

            RunMessage::ShowDumpResult => {
                self.show_dump_result = true;
                Task::none()
            }
            RunMessage::HideDumpResult => {
                self.show_dump_result = false;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, RunMessage> {
        let tail = if self.result.is_some() && self.result.as_ref().unwrap().is_ok() {
            row![
                space::horizontal(),
                button("差异统计")
                    .style(button::secondary)
                    .on_press(RunMessage::Dump(DumpType::Statis)),
                space().width(SPACING),
                button("导出文件")
                    .style(button::secondary)
                    .on_press(RunMessage::Dump(DumpType::Result)),
                space().width(4. * SPACING),
                button("退出").on_press(RunMessage::HideResult),
                space().width(4. * SPACING)
            ]
        } else {
            row![
                space::horizontal(),
                button("退出").on_press(RunMessage::HideResult),
                space().width(4. * SPACING)
            ]
        };
        let content: Element<_> = if let Some(result) = self.result.as_ref() {
            match result.as_ref() {
                Ok((freight_report, customs_report)) => column![
                    text("运费对账结果").width(Fill).style(text::primary),
                    container(dataframe_table(freight_report))
                        .padding(CONTAINER_PADDING)
                        .style(container::rounded_box),
                    text("报关费对账结果").width(Fill).style(text::primary),
                    container(dataframe_table(customs_report))
                        .padding(CONTAINER_PADDING)
                        .style(container::rounded_box),
                ]
                .into(),
                Err(error) => column![
                    text("出错了！！！").style(text::danger),
                    "错误信息:",
                    text!("{}", error)
                ]
                .spacing(SPACING)
                .into(),
            }
        } else {
            text("这不应该出现，因为显示还没有任何结果").into()
        };
        let content: Element<_> = container(column![
            text!("对账结果")
                .size(H1_SIZE)
                .center()
                .style(text::success),
            space().height(SPACING),
            scrollable(scrollable(content).horizontal().height(Fill)).height(Fill),
            tail
        ])
        .width(Fill)
        .height(Fill)
        .padding(CONTAINER_PADDING)
        .style(container::rounded_box)
        .into();
        if self.show_dump_result && self.dump_result.is_some() {
            modal(content, self.dump_view(), RunMessage::HideDumpResult)
        } else {
            content
        }
    }

    fn dump_view(&self) -> Element<'_, RunMessage> {
        if let Some(res) = self.dump_result.as_ref() {
            match res.as_ref() {
                Ok(msg) => text(msg).into(),
                Err(msg) => text!("{}", msg).into(),
            }
        } else {
            text!("这个界面属于实现错误，请检查代码逻辑").into()
        }
    }
}

impl Runner {
    pub async fn run(data: [Vec<UserData>; 2]) -> RunResult {
        use crate::ParserType;
        use logirecon_core::reconsile::{CUSTOMS_RECONSILE_COLUMNS, FREIGHT_RECONSILE_COLUMNS};
        use logirecon_core::{
            DDDParser, DataRepo, Error, HeadwayParser, Parse, ReconsileOption, TSParser, WBParser,
        };
        // 模拟UI返回的用户输入结构体
        let [bills, shipments] = data;
        if bills.is_empty() {
            return Err(Error::Process(
                "物流账单内未检测到任何数据，请检查是否导入文件或者核对表头是否正确".into(),
            ));
        }
        if shipments.is_empty() {
            return Err(Error::Process(
                "我方明细内未检测到任何数据，请检查是否导入文件或者核对表头是否正确".into(),
            ));
        }

        // parse
        let bills: Vec<_> = bills
            .into_iter()
            .filter_map(|data| {
                let mut parser: Box<dyn Parse<BillValidated>> = match data.parser_type {
                    ParserType::WB => Box::new(WBParser::default()),
                    ParserType::DDD => Box::new(DDDParser::default()),
                    ParserType::TS => Box::new(TSParser::default()),
                    ParserType::Headway => return None,
                    ParserType::GRT => {
                        let mut parser = Box::new(WBParser::default());
                        parser.forwarder = "国润通".into();
                        parser
                    }
                };
                parser
                    .provider_mut()
                    .update_headers(data.headers)
                    .with_primary(data.primary);
                for (path, sheet) in data.sheets {
                    parser.provider_mut().add_sheets(path, sheet);
                }
                Some(parser.parse().map_err(|e| {
                    Error::Process(format!(
                        "\n解析\"{}\"时出现错误, {}",
                        data.parser_type.name(),
                        e
                    ))
                }))
            })
            .collect::<Result<_, _>>()?;
        let shipments: Vec<_> = shipments
            .into_iter()
            .filter_map(|data| {
                let mut parser: Box<dyn Parse<ShipmentValidated>> = match data.parser_type {
                    ParserType::Headway => Box::new(HeadwayParser::default()),
                    _ => return None,
                };
                parser
                    .provider_mut()
                    .update_headers(data.headers)
                    .with_primary(data.primary);
                for (path, sheet) in data.sheets {
                    parser.provider_mut().add_sheets(path, sheet);
                }
                Some(parser.parse().map_err(|e| {
                    Error::Process(format!(
                        "\n解析{}时出现错误, {}",
                        data.parser_type.name(),
                        e
                    ))
                }))
            })
            .collect::<Result<_, _>>()?;

        // user input 解析
        let repo = DataRepo::new(bills, shipments)?;
        // 获取运单和报关单的差异分析报表
        let (freight_bill, freight_self) = repo.get_freight()?;
        let freight_report = ReconsileOption::new_with_columns(FREIGHT_RECONSILE_COLUMNS)
            .left(freight_bill, "物流")
            .right(freight_self, "我方")
            .try_into_reconsiler()?
            .build_result()?
            .get_long_result()?;
        let (customs_bill, customs_self) = repo.get_customs()?;
        let customs_report = ReconsileOption::new_with_columns(CUSTOMS_RECONSILE_COLUMNS)
            .left(customs_bill, "物流")
            .right(customs_self, "我方")
            .try_into_reconsiler()?
            .build_result()?
            .get_long_result()?;

        Ok((freight_report, customs_report))
    }

    pub async fn dump_to_excel(freight: DataFrame, customs: DataFrame) -> DumpResult {
        use polars_excel_writer::PolarsExcelWriter;
        use rfd::AsyncFileDialog;
        if let Some(path) = AsyncFileDialog::new()
            .set_title("导出文件")
            .set_file_name("物流对账明细.xlsx")
            .add_filter("excel", &EXCEL_SUFFIX)
            .save_file()
            .await
        {
            let mut wb = PolarsExcelWriter::new();
            wb.set_worksheet_name("运费对比结果")?;
            wb.write_dataframe(&freight)?;
            wb.add_worksheet();
            wb.set_worksheet_name("报关费对比结果")?;
            wb.write_dataframe(&customs)?;
            wb.save(path.path())?;
            Ok(format!("已保存到 {}", path.file_name()))
        } else {
            Ok("取消保存".to_string())
        }
    }

    pub async fn dump_statis_to_excel(freight: DataFrame, customs: DataFrame) -> DumpResult {
        use polars::prelude::*;
        use rfd::AsyncFileDialog;
        if let Some(path) = AsyncFileDialog::new()
            .set_title("导出文件")
            .set_file_name("账单差异统计.xlsx")
            .add_filter("excel", &EXCEL_SUFFIX)
            .save_file()
            .await
        {
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
            {
                let mut file = std::fs::File::create("core/data/test/output.csv").unwrap();
                CsvWriter::new(&mut file).finish(&mut df.clone())?;
            }
            {
                use polars_excel_writer::PolarsExcelWriter;
                let mut wb = PolarsExcelWriter::new();
                wb.set_worksheet_name("差异结果统计")?;
                wb.write_dataframe(&df)?;
                wb.save(path.path())?;
                Ok(format!("已保存到 {}", path.file_name()))
            }
        } else {
            Ok("取消导出".to_string())
        }
    }
}
