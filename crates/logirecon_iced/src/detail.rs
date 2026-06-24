use crate::constants::{H2_SIZE, PADDING, SPACING};

use super::template::{self, TemplateType};
use iced::{Element, Task, widget::combo_box};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub title: String,
    pub template_selection: ComboState,
    pub temp_type: Option<TemplateType>,
    pub templates: Vec<template::State>,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddTemplate,
    SelectTemplateType(TemplateType),
    Template(usize, Box<template::Message>),
}

impl State {
    pub fn bill() -> Self {
        Self {
            title: "物流账单".into(),
            template_selection: vec![
                TemplateType::WB,
                TemplateType::TS,
                TemplateType::Ddd,
                TemplateType::Grt,
            ]
            .into(),
            temp_type: None,
            templates: vec![],
        }
    }

    pub fn shipment() -> Self {
        Self {
            title: "我方明细".into(),
            template_selection: vec![TemplateType::Headway].into(),
            temp_type: None,
            templates: vec![],
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectTemplateType(t) => {
                self.temp_type = Some(t);
                Task::none()
            }
            Message::AddTemplate => {
                if let Some(temp) = self.temp_type.as_ref() {
                    let state = template::State::new(temp.clone());
                    self.templates.push(state);
                };
                Task::none()
            }
            Message::Template(i, message) => {
                let message = *message;
                match message {
                    template::Message::Delete => {
                        self.templates.remove(i);
                        Task::none()
                    }
                    _ => {
                        if let Some(state) = self.templates.get_mut(i) {
                            state
                                .update(message)
                                .map(move |m| Message::Template(i, Box::new(m)))
                        } else {
                            Task::none()
                        }
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::{
            Alignment::*,
            Border,
            Length::*,
            widget::{button, column, container, keyed_column, row, scrollable, text},
        };
        let title = text!("{}", &self.title)
            .width(Fill)
            .align_x(Center)
            .size(H2_SIZE);
        let header = row![
            combo_box(
                &self.template_selection.0,
                "选择模板类型",
                self.temp_type.as_ref(),
                Message::SelectTemplateType
            ),
            button("添加模板").on_press(Message::AddTemplate)
        ];
        let body: Element<_> = if !self.templates.is_empty() {
            keyed_column(self.templates.iter().enumerate().map(|(i, t)| {
                (
                    t.id,
                    t.view().map(move |m| Message::Template(i, Box::new(m))),
                )
            }))
            .spacing(SPACING)
            .into()
        } else {
            text!("点击按钮添加模板...")
                .center()
                .width(Fill)
                .style(text::secondary)
                .into()
        };
        container(
            column![title, header, scrollable(body)]
                .align_x(Center)
                .width(Fill)
                .spacing(SPACING),
        )
        .center_x(Fill)
        .height(Fill)
        .padding(PADDING)
        .style(|theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: None,
                text_color: Some(palette.background.weakest.text),
                border: Border {
                    width: 2.0,
                    radius: 10.0.into(),
                    color: palette.background.weak.color,
                },
                ..Default::default()
            }
        })
        .into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "Vec<TemplateType>", into = "Vec<TemplateType>")]
pub struct ComboState(combo_box::State<TemplateType>);

impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<Vec<TemplateType>> for ComboState {
    fn from(value: Vec<TemplateType>) -> Self {
        Self(combo_box::State::new(value))
    }
}

impl From<ComboState> for Vec<TemplateType> {
    fn from(value: ComboState) -> Self {
        value.0.into_options()
    }
}
