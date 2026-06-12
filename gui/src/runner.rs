use super::{UserData, dataframe_table};
use crate::{CONTAINER_PADDING, EXCEL_SUFFIX, H1_SIZE, SPACING, modal};
use iced::{
    Element,
    Length::*,
    Task,
    widget::{button, column, container, row, scrollable, space, text},
};
use polars::prelude::DataFrame;
use std::fmt::Debug;

type RunResult = Result<(DataFrame, DataFrame), logirecon_core::Error>;

#[derive(Debug, Clone)]
pub enum RunMessage {
    Run,
    UpdateResult(RunResult),
    ShowResult,
    HideResult,

    DumpResult,
    Dumped(Result<String, logirecon_core::Error>),
    ShowDumpResult,
    HideDumpResult,
}

/// 用于展示用户输入的默认运行组件
#[derive(Debug, Clone, Default)]
pub struct Runner {
    pub show_result: bool,
    pub running: bool,
    pub result: Option<RunResult>,

    pub dump_result: Option<Result<String, logirecon_core::Error>>,
    pub show_dump_result: bool,
    pub dumpping: bool,
}

impl Runner {
    pub fn update(&mut self, message: RunMessage, details: [Vec<UserData>; 2]) -> Task<RunMessage> {
        match message {
            RunMessage::Run => {
                if !self.running {
                    self.result = None;
                    self.running = false;
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

            RunMessage::DumpResult => {
                if let Some(res) = self.result.as_ref()
                    && !self.dumpping
                {
                    if let Ok((freight, customs)) = res.as_ref() {
                        self.dumpping = true;
                        self.dump_result = None;
                        Task::perform(
                            Self::dump_to_excel(freight.clone(), customs.clone()),
                            RunMessage::Dumped,
                        )
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }

            RunMessage::Dumped(res) => {
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
                button("导出文件")
                    .style(button::secondary)
                    .on_press(RunMessage::DumpResult),
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
            scrollable(content).height(Fill),
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
        use logirecon_core::{DataRepo, Error, HeadwayParser, Parse, ReconsileOption, WBParser};
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
        let bills = bills.into_iter().filter_map(|data| {
            let mut parser = match data.parser_type {
                ParserType::WB => WBParser::default(),
                _ => return None,
            };
            parser
                .provider_mut()
                .update_headers(data.headers)
                .with_primary(data.primary);
            for (path, sheet) in data.sheets {
                parser.provider_mut().add_sheets(path, sheet);
            }
            parser.parse().ok()
        });
        let shipments = shipments.into_iter().filter_map(|data| {
            let mut parser = match data.parser_type {
                ParserType::Headway => HeadwayParser::default(),
                _ => return None,
            };
            parser
                .provider_mut()
                .update_headers(data.headers)
                .with_primary(data.primary);
            for (path, sheet) in data.sheets {
                parser.provider_mut().add_sheets(path, sheet);
            }
            parser.parse().ok()
        });

        // user input 解析
        let repo = DataRepo::new(bills, shipments)
            .map_err(|_| Error::Process("表格解析出现问题，请核对表格是否正确".to_string()))?;
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

    pub async fn dump_to_excel(
        freight: DataFrame,
        customs: DataFrame,
    ) -> Result<String, logirecon_core::Error> {
        use polars_excel_writer::PolarsExcelWriter;
        use rfd::AsyncFileDialog;
        if let Some(path) = AsyncFileDialog::new()
            .set_title("导入文件")
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
}
