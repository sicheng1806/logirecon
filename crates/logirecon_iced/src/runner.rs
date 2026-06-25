use iced::{Element, Task};
use logirecon::DataFrame;
use logirecon::parser::{
    AsHeaders, DDDParseConfig, HeadwayParseConfig, TSParseConfig, WBParseConfig,
};

use crate::components::{dataframe_table, modal};
use crate::constants::{EXCEL_SUFFIX, H1_SIZE, PADDING, SPACING};
use crate::modal::ModalView;
use crate::{sheet, template};

use logirecon::runner::{
    ParseConfig, ReadConfig, RunError, Template, get_reconciler, stasis_freight_and_customs,
};

#[derive(Debug, Clone)]
pub struct State {
    running: bool,
    result: Option<Result<(DataFrame, DataFrame), String>>,
    msg: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Run([Vec<template::State>; 2]),
    Ran(Result<(DataFrame, DataFrame), String>),
    ExportToExcel,
    ExportStasisToExcel,
    Exported(Result<String, String>),
    ShowModal(bool),
    ShowMsg(String),
    HideMsg,
}

impl State {
    pub fn new() -> Self {
        Self {
            running: false,
            result: None,
            msg: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowModal(_) => Task::none(),
            Message::Run(data) => {
                self.running = true;
                Task::perform(run(data), |res| {
                    Message::Ran(res.map_err(|e| format!("{e}")))
                })
            }
            Message::Ran(res) => {
                self.running = false;
                self.result = Some(res);
                Task::done(Message::ShowModal(true))
            }
            Message::ExportToExcel => {
                if !self.running
                    && let Some(Ok((freight, customs))) = self.result.as_ref()
                {
                    Task::perform(
                        export_result_to_excel(freight.clone(), customs.clone()),
                        |res| Message::Exported(res.map_err(|e| e.to_string())),
                    )
                } else {
                    Task::none()
                }
            }
            Message::ExportStasisToExcel => {
                if !self.running
                    && let Some(Ok((freight, customs))) = self.result.as_ref()
                {
                    Task::perform(
                        export_stasis_to_excel(freight.clone(), customs.clone()),
                        |res| Message::Exported(res.map_err(|e| e.to_string())),
                    )
                } else {
                    Task::none()
                }
            }
            Message::Exported(res) => match res {
                Ok(msg) => Task::done(Message::ShowMsg(msg)),
                Err(msg) => Task::done(Message::ShowMsg(format!("出错了: {}", msg))),
            },
            Message::ShowMsg(msg) => {
                self.msg = Some(msg);
                Task::none()
            }
            Message::HideMsg => {
                self.msg = None;
                Task::none()
            }
        }
    }
}

impl ModalView for State {
    type Message = Message;

    fn modal_view(&self) -> iced::Element<'_, Self::Message> {
        use iced::{
            Length::*,
            widget::{button, column, container, row, scrollable, space, text},
        };
        let footer = if self.result.is_some() && self.result.as_ref().unwrap().is_ok() {
            row![
                space::horizontal(),
                button("差异统计")
                    .style(button::secondary)
                    .on_press(Message::ExportStasisToExcel),
                space().width(SPACING),
                button("导出文件")
                    .style(button::secondary)
                    .on_press(Message::ExportToExcel),
                space().width(4. * SPACING),
                button("退出").on_press(Message::ShowModal(false)),
                space().width(4. * SPACING)
            ]
        } else {
            row![
                space::horizontal(),
                button("退出").on_press(Message::ShowModal(false)),
                space().width(4. * SPACING)
            ]
        };
        let content: Element<_> = if let Some(result) = self.result.as_ref() {
            match result.as_ref() {
                Ok((freight_report, customs_report)) => column![
                    text("运费对账结果").width(Fill).style(text::primary),
                    container(dataframe_table(freight_report))
                        .padding(PADDING)
                        .style(container::rounded_box),
                    text("报关费对账结果").width(Fill).style(text::primary),
                    container(dataframe_table(customs_report))
                        .padding(PADDING)
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
            footer
        ])
        .width(Fill)
        .height(Fill)
        .padding(PADDING)
        .style(container::rounded_box)
        .into();
        if let Some(msg) = self.msg.as_ref() {
            modal(content, text!("{}", msg).center(), Message::HideMsg)
        } else {
            content
        }
    }
}

impl From<template::State> for Template {
    fn from(value: template::State) -> Self {
        use crate::template::TemplateType;
        let template::State {
            temp_type,
            headers,
            sheets,
            ..
        } = value;
        let parse_config = match temp_type {
            TemplateType::WB => {
                let mut config = WBParseConfig::default();
                config.headers.update_headers(headers);
                ParseConfig::WB(config)
            }
            TemplateType::Grt => {
                let mut config = WBParseConfig::grt();
                config.headers.update_headers(headers);
                ParseConfig::WB(config)
            }
            TemplateType::TS => {
                let mut config = TSParseConfig::default();
                config.headers.update_headers(headers);
                ParseConfig::TS(config)
            }
            TemplateType::Ddd => {
                let mut config = DDDParseConfig::default();
                config.headers.update_headers(headers);
                ParseConfig::DDD(config)
            }
            TemplateType::Headway => {
                let mut config = HeadwayParseConfig::default();
                config.headers.update_headers(headers);
                ParseConfig::Headway(config)
            }
        };
        let sources = sheets
            .into_iter()
            .filter_map(
                |sheet::State {
                     chosen,
                     selected,
                     path,
                     ..
                 }| {
                    if chosen && let Some(name) = selected {
                        Some(ReadConfig::ExcelFilePath { path, name })
                    } else {
                        None
                    }
                },
            )
            .collect();
        Template {
            parse_config,
            sources,
        }
    }
}

async fn run(data: [Vec<template::State>; 2]) -> Result<(DataFrame, DataFrame), RunError> {
    let [bills, shipments] = data;
    if bills.is_empty() {
        return Err(RunError::Any(
            "物流账单内未检测到任何数据，请检查是否导入文件或者核对表头是否正确".into(),
        ));
    }
    if shipments.is_empty() {
        return Err(RunError::Any(
            "我方明细内未检测到任何数据，请检查是否导入文件或者核对表头是否正确".into(),
        ));
    }

    let templates: Vec<Template> = bills
        .into_iter()
        .chain(shipments)
        .map(|t| t.into())
        .collect();
    let (freight_reconciler, customs_reconciler) = get_reconciler(templates)?;
    let freight = freight_reconciler.get_long_result()?;
    let customs = customs_reconciler.get_long_result()?;
    Ok((freight, customs))
}

async fn export_result_to_excel(
    freight: DataFrame,
    customs: DataFrame,
) -> Result<String, RunError> {
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
        Ok(format!("保存到 {}", path.file_name()))
    } else {
        Ok("取消保存".to_string())
    }
}

async fn export_stasis_to_excel(
    freight: DataFrame,
    customs: DataFrame,
) -> Result<String, RunError> {
    use polars_excel_writer::PolarsExcelWriter;
    use rfd::AsyncFileDialog;
    if let Some(path) = AsyncFileDialog::new()
        .set_title("导出文件")
        .set_file_name("账单差异统计.xlsx")
        .add_filter("excel", &EXCEL_SUFFIX)
        .save_file()
        .await
    {
        let stasis = stasis_freight_and_customs(freight, customs)?;
        if stasis.height() > 0 {
            let mut wb = PolarsExcelWriter::new();
            wb.set_worksheet_name("差异结果统计")?;
            wb.write_dataframe(&stasis)?;
            wb.save(path.path())?;
            Ok(format!("保存到 {}", path.file_name()))
        } else {
            Ok("没有差异行".to_string())
        }
    } else {
        Ok("取消保存".to_string())
    }
}
