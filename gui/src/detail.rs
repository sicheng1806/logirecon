use super::template::{Template, TemplateMessage};
use super::{CONTAINER_PADDING, H2_SIZE, ParserType, SPACING};

use iced::{
    Alignment::*,
    Border, Element,
    Length::*,
    Task,
    widget::{button, column, combo_box, container, keyed_column, row, scrollable, text},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(from = "Vec<ParserType>", into = "Vec<ParserType>")]
pub struct TemplateState(combo_box::State<ParserType>);

impl From<Vec<ParserType>> for TemplateState {
    fn from(value: Vec<ParserType>) -> Self {
        Self(combo_box::State::new(value))
    }
}

impl From<TemplateState> for Vec<ParserType> {
    fn from(value: TemplateState) -> Self {
        value.0.into_options()
    }
}

/// 组织模板组件的容器
///
/// # Note
///
/// 模板的表头编辑视图需手动实现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillDetails {
    pub title: String,
    pub avaliable_templates: TemplateState,
    pub current_template: Option<ParserType>,
    pub templates: Vec<Template>,
}

#[derive(Debug, Clone)]
pub enum DetailMessage {
    AddTemplate,
    SelectTemplate(ParserType),
    TempMessage(usize, Box<TemplateMessage>),
}

impl BillDetails {
    pub fn update(&mut self, message: DetailMessage) -> Task<DetailMessage> {
        match message {
            DetailMessage::SelectTemplate(t) => {
                self.current_template = Some(t);
                Task::none()
            }
            DetailMessage::AddTemplate => {
                if let Some(temp) = self.current_template.clone() {
                    let temp = Template::new(temp);
                    self.templates.push(temp);
                };
                Task::none()
            }
            DetailMessage::TempMessage(i, temp_message) => {
                if let TemplateMessage::Delete = *temp_message {
                    self.templates.remove(i);
                    Task::none()
                } else if let Some(template) = self.templates.get_mut(i) {
                    template
                        .update(*temp_message)
                        .map(move |m| DetailMessage::TempMessage(i, Box::new(m)))
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, DetailMessage> {
        let title = text!("{}", &self.title)
            .width(Fill)
            .align_x(Center)
            .size(H2_SIZE);
        let header = row![
            combo_box(
                &self.avaliable_templates.0,
                "选择模板类型",
                self.current_template.as_ref(),
                DetailMessage::SelectTemplate,
            ),
            button("添加模板").on_press(DetailMessage::AddTemplate)
        ];
        let contents: Element<_> = if !self.templates.is_empty() {
            keyed_column(self.templates.iter().enumerate().map(|(i, t)| {
                (
                    t.id,
                    t.view()
                        .map(move |m| DetailMessage::TempMessage(i, Box::new(m))),
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
            column![title, header, scrollable(contents)]
                .align_x(Center)
                .width(Fill)
                .spacing(SPACING),
        )
        .center_x(Fill)
        .height(Fill)
        .padding(CONTAINER_PADDING)
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
