use super::sheet::{Sheet, SheetMessage};
use crate::{CONTAINER_PADDING, EXCEL_SUFFIX, H1_SIZE, SPACING};

use super::{ParserType, svg_button};
use iced::{
    Alignment::*,
    Border, Element, Function,
    Length::*,
    Task,
    widget::{
        button, column, container, keyed_column, row, rule, scrollable, space, text, text_input,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 表单模板组件
///
/// 根据模板导入工作簿的组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: uuid::Uuid,
    pub show_header_view: bool,
    pub parser_type: ParserType,
    pub headers: HashMap<String, String>,
    pub sheets: Vec<Sheet>,
}

#[derive(Debug, Clone)]
pub enum TemplateMessage {
    // 外部信息
    Delete,
    ShowHeaderView,
    HideHeaderView,
    UpdateHeader(String, String),
    ImportFile,
    InsertSheet(Option<Sheet>),
    SheetMessage(usize, SheetMessage),
}

impl Template {
    pub fn new(temp_type: ParserType) -> Self {
        use logirecon_core::{DDDParser, HeadwayParser, TSParser, WBParser};
        let headers: HashMap<String, String> = match temp_type {
            ParserType::WB => HashMap::from_iter(
                WBParser::DEFAULT_HEADERS.map(|t| (t.to_string(), t.to_string())),
            ),
            ParserType::Headway => HashMap::from_iter(
                HeadwayParser::DEFAULT_HEADERS.map(|t| (t.to_string(), t.to_string())),
            ),
            ParserType::TS => HashMap::from_iter(
                TSParser::DEFAULT_HEADERS.map(|t| (t.to_string(), t.to_string())),
            ),
            ParserType::DDD => HashMap::from_iter(
                DDDParser::DEFAULT_HEADERS.map(|t| (t.to_string(), t.to_string())),
            ),
            ParserType::GRT => {
                let mut headers = HashMap::from_iter(
                    WBParser::DEFAULT_HEADERS.map(|t| (t.to_string(), t.to_string())),
                );
                headers.insert("订单号".to_string(), "扩展单号".to_string());
                headers.insert("仓库编码".to_string(), "地址编码".to_string());
                headers
            }
        };
        Self {
            id: uuid::Uuid::new_v4(),
            parser_type: temp_type,
            headers,
            sheets: vec![],
            show_header_view: false,
        }
    }

    async fn load_sheet_file() -> Option<Sheet> {
        if let Some(path) = rfd::AsyncFileDialog::new()
            .add_filter("excel", &EXCEL_SUFFIX)
            .set_directory("/")
            .pick_file()
            .await
        {
            Sheet::new_from_path(path)
        } else {
            None
        }
    }
}

impl Template {
    pub fn update(&mut self, message: TemplateMessage) -> Task<TemplateMessage> {
        match message {
            TemplateMessage::Delete => Task::none(),
            TemplateMessage::ShowHeaderView => {
                self.show_header_view = true;
                Task::none()
            }
            TemplateMessage::HideHeaderView => {
                self.show_header_view = false;
                Task::none()
            }
            TemplateMessage::UpdateHeader(k, v) => {
                if let Some(header) = self.headers.get_mut(&k) {
                    *header = v;
                };
                Task::none()
            }
            TemplateMessage::ImportFile => Task::perform(Self::load_sheet_file(), |sheet| {
                TemplateMessage::InsertSheet(sheet)
            }),
            TemplateMessage::InsertSheet(sheet) => {
                if let Some(sheet) = sheet {
                    self.sheets.push(sheet);
                };
                Task::none()
            }
            TemplateMessage::SheetMessage(i, SheetMessage::Delete) => {
                self.sheets.remove(i);
                Task::none()
            }
            TemplateMessage::SheetMessage(i, sheet_message) => {
                if let Some(sheet) = self.sheets.get_mut(i) {
                    sheet.update(sheet_message);
                    Task::none()
                } else {
                    Task::none()
                }
            }
        }
    }
    /// 默认视图
    ///
    /// 提供模板卡片视图
    pub fn view(&self) -> Element<'_, TemplateMessage> {
        let title = self.parser_type.name();
        let header = row![
            space().width(SPACING * 2.),
            text!("{title}").align_x(Center),
            space().width(SPACING * 2.),
            space::horizontal(),
            button("编辑表头")
                .style(button::secondary)
                .on_press(TemplateMessage::ShowHeaderView),
            button("导入文件")
                .style(button::secondary)
                .on_press(TemplateMessage::ImportFile),
            svg_button("/public/cancel.svg").on_press(TemplateMessage::Delete),
        ]
        .align_y(Center)
        .spacing(SPACING)
        .width(Fill);
        let sheets: Element<_> = if !self.sheets.is_empty() {
            keyed_column(self.sheets.iter().enumerate().map(|(i, sheet)| {
                (
                    sheet.id,
                    sheet.view().map(TemplateMessage::SheetMessage.with(i)),
                )
            }))
            .into()
        } else {
            text("请先导入文件...")
                .style(text::secondary)
                .center()
                .width(Fill)
                .into()
        };
        let body = scrollable(sheets);

        let content: Element<_> = container(column![
            header,
            space().height(6),
            rule::horizontal(2),
            space().height(SPACING),
            body,
        ])
        .padding([SPACING, CONTAINER_PADDING])
        .style(|theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weakest.color.into()),
                text_color: Some(palette.background.weakest.text),
                border: Border {
                    width: 1.0,
                    radius: 20.0.into(),
                    color: palette.background.weak.color,
                },
                ..Default::default()
            }
        })
        .into();
        content
    }

    /// 表头编辑视图
    ///
    /// 提供表头编辑的模态框内容
    pub fn header_view(&self) -> Element<'_, TemplateMessage> {
        let content: Element<_> = if !self.headers.is_empty() {
            let title = text!("{}", &self.parser_type.name())
                .size(H1_SIZE)
                .width(Fill)
                .align_x(Center);
            let headers = row(self.headers.iter().map(|(k, v)| {
                column![
                    text!(" {}", k).style(text::primary),
                    text_input("", v).on_input(|new_header| TemplateMessage::UpdateHeader(
                        k.to_owned(),
                        new_header
                    ))
                ]
                .align_x(Start)
                .spacing(SPACING)
                .into()
            }))
            .align_y(Center);
            column![title, space().height(4. * SPACING), headers].into()
        } else {
            text!("出现这个说明没有表头，程序的实现有问题(-_-)").into()
        };

        let tail = row![
            space::horizontal(),
            button("完成").on_press(TemplateMessage::HideHeaderView),
            space().width(CONTAINER_PADDING * 2.)
        ];

        container(column![content, space::vertical(), tail])
            .center(Fill)
            .padding(5. * CONTAINER_PADDING)
            .style(container::rounded_box)
            .into()
    }
}
