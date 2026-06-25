//! 数据模板组件
//!
//! 有若干 [sheet::State] 组成

use std::collections::HashMap;

use iced::{Element, Function, Task};
use logirecon::parser::{
    AsHeaders, DddParseConfig, HeadwayParseConfig, JydParseConfig, TsParseConfig, WBParseConfig,
};
use serde::{Deserialize, Serialize};

use crate::{
    components::cancel_button,
    constants::{EXCEL_SUFFIX, H1_SIZE, PADDING, SPACING, STROKE_BOLD},
    modal::ModalView,
};

use super::sheet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateType {
    Wb,
    Grt,
    Jm,
    Ddd,
    Ts,
    Jyd,
    Headway,
}

impl TemplateType {
    pub fn name(&self) -> String {
        match self {
            Self::Wb => "万邦",
            Self::Grt => "国润通",
            Self::Jm => "积米",
            Self::Ddd => "嘀嗒嘀",
            Self::Ts => "天盛",
            Self::Jyd => "京奕达",
            Self::Headway => "头程明细",
        }
        .into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub id: uuid::Uuid,
    pub temp_type: TemplateType,
    pub headers: HashMap<String, String>,
    pub sheets: Vec<sheet::State>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Delete,
    ShowModal(bool),
    UpdateHeader(String, String),
    ImportFile,
    InsertSheet(Option<sheet::State>),
    Sheet(usize, sheet::Message),
}

impl State {
    pub fn new(temp_type: TemplateType) -> Self {
        let headers = match &temp_type {
            TemplateType::Wb => WBParseConfig::default().headers.as_headers(),
            TemplateType::Grt => WBParseConfig::grt().headers.as_headers(),
            TemplateType::Jm => WBParseConfig::jm().headers.as_headers(),
            TemplateType::Ts => TsParseConfig::default().headers.as_headers(),
            TemplateType::Ddd => DddParseConfig::default().headers.as_headers(),
            TemplateType::Jyd => JydParseConfig::default().headers.as_headers(),
            TemplateType::Headway => HeadwayParseConfig::default().headers.as_headers(),
        };
        Self {
            id: uuid::Uuid::new_v4(),
            temp_type,
            headers,
            sheets: vec![],
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Delete | Message::ShowModal(_) => Task::none(),
            Message::UpdateHeader(k, v) => {
                if let Some(header) = self.headers.get_mut(&k) {
                    *header = v;
                };
                Task::none()
            }
            Message::ImportFile => Task::perform(load_workbook(), Message::InsertSheet),
            Message::InsertSheet(sheet) => {
                if let Some(sheet) = sheet {
                    self.sheets.push(sheet);
                };
                Task::none()
            }
            Message::Sheet(i, sheet::Message::Delete) => {
                self.sheets.remove(i);
                Task::none()
            }
            Message::Sheet(i, message) => {
                if let Some(state) = self.sheets.get_mut(i) {
                    state.update(message);
                };
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::{
            Alignment::*,
            Border,
            Length::*,
            widget::{button, column, container, keyed_column, row, rule, scrollable, space, text},
        };
        let header = row![
            space().width(SPACING * 2.),
            text!("{}", self.temp_type.name()).style(text::primary),
            space().width(SPACING * 2.),
            space::horizontal(),
            button("编辑表头")
                .style(button::secondary)
                .on_press(Message::ShowModal(true)),
            button("导入文件")
                .style(button::secondary)
                .on_press(Message::ImportFile),
            cancel_button().on_press(Message::Delete),
        ]
        .align_y(Center)
        .spacing(SPACING)
        .width(Fill);

        let body: Element<_> = if !self.sheets.is_empty() {
            keyed_column(
                self.sheets
                    .iter()
                    .enumerate()
                    .map(|(i, sheet)| (sheet.id, sheet.view().map(Message::Sheet.with(i)))),
            )
            .into()
        } else {
            text("请先导入文件...")
                .style(text::secondary)
                .center()
                .width(Fill)
                .into()
        };
        container(column![
            header,
            space().height(SPACING * 0.5),
            rule::horizontal(STROKE_BOLD),
            space().height(SPACING),
            scrollable(body),
        ])
        .padding([SPACING, PADDING])
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
        .into()
    }
}

impl ModalView for State {
    type Message = Message;

    fn modal_view(&self) -> Element<'_, Self::Message> {
        use iced::{
            Alignment::*,
            Length::*,
            widget::{button, column, container, row, space, text, text_input},
        };
        let footer = row![
            space::horizontal(),
            button("完成").on_press(Message::ShowModal(false)),
            space().width(PADDING * 2.)
        ];
        let body: Element<_> = if !self.headers.is_empty() {
            let header = text!("{}", self.temp_type.name())
                .size(H1_SIZE)
                .width(Fill)
                .align_x(Center);
            let body = row(self.headers.iter().map(|(k, v)| {
                column![
                    text!(" {}", k).style(text::primary),
                    text_input("", v).on_input(Message::UpdateHeader.with(k.to_owned()))
                ]
                .align_x(Start)
                .spacing(SPACING)
                .into()
            }))
            .align_y(Center);
            column![header, space().height(4. * SPACING), body].into()
        } else {
            text!("出现这个说明没有表头，程序的实现有问题(0_-)").into()
        };
        container(column![body, space::vertical(), footer])
            .center(Fill)
            .padding(5. * PADDING)
            .style(container::rounded_box)
            .into()
    }
}

async fn load_workbook() -> Option<sheet::State> {
    if let Some(path) = rfd::AsyncFileDialog::new()
        .add_filter("excel", &EXCEL_SUFFIX)
        .pick_file()
        .await
    {
        sheet::State::new(path).ok()
    } else {
        None
    }
}
