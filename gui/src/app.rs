use crate::{CONTAINER_PADDING, CONTAINER_SPACING, H1_SIZE, SPACING};
use std::{fmt::Debug, path::PathBuf};

use super::{
    ParserType, UserData,
    detail::{BillDetails, DetailMessage},
    modal,
    runner::{RunMessage, Runner},
    template::{Template, TemplateMessage},
};

use iced::{
    Alignment::*,
    Element, Function,
    Length::*,
    Program, Subscription, Task, Theme,
    widget::{button, center, column, container, row, space, text},
};

use serde::{Deserialize, Serialize};

pub fn application() -> iced::Application<impl Program<Message = AppMessage, Theme = Theme>> {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .exit_on_close_request(false)
}

#[derive(Debug, Clone)]
pub enum LoadError {
    File,
    Format,
}
#[derive(Debug, Clone)]
pub enum SaveError {
    Format,
    Write,
}

pub enum App {
    Loading,
    Loaded(Box<State>),
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Loaded(Box<Result<SaveState, LoadError>>),
    Message(Message),
}

#[derive(Debug, Clone)]
pub struct State {
    exiting: bool,
    details: [BillDetails; 2],
    runner: Runner,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SaveState {
    pub details: [BillDetails; 2],
}

#[derive(Debug, Clone)]
pub enum Message {
    DetailMessage(usize, DetailMessage),
    RunMessage(RunMessage),
    Reset,
    Save,
    Saved(Result<(), SaveError>),
}

impl App {
    pub fn new() -> (Self, Task<AppMessage>) {
        (
            Self::Loading,
            Task::perform(SaveState::load(), |res| AppMessage::Loaded(Box::new(res))),
        )
    }

    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match self {
            Self::Loading => match message {
                AppMessage::Loaded(res) => {
                    match *res {
                        Ok(state) => {
                            *self = App::Loaded(Box::new(State {
                                details: state.details,
                                ..Default::default()
                            }))
                        }
                        Err(_) => *self = App::Loaded(Box::default()),
                    }
                    Task::none()
                }
                _ => Task::none(),
            },
            Self::Loaded(state) => match message {
                AppMessage::Loaded(_) => Task::none(),
                AppMessage::Message(message) => state.update(message).map(AppMessage::Message),
            },
        }
    }

    pub fn view(&self) -> Element<'_, AppMessage> {
        match self {
            Self::Loading => center("数据载入中...").into(),
            Self::Loaded(state) => state.view().map(AppMessage::Message),
        }
    }

    pub fn subscription(&self) -> Subscription<AppMessage> {
        use iced::window;

        window::close_requests().filter_map(|_window_id| Some(AppMessage::Message(Message::Save)))
    }
}

impl State {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Save => {
                if !self.exiting {
                    self.exiting = true;
                    Task::perform(
                        SaveState {
                            details: self.details.clone(),
                        }
                        .save(),
                        Message::Saved,
                    )
                } else {
                    Task::none()
                }
            }
            Message::Saved(res) => {
                self.exiting = false;
                if let Err(error) = res {
                    println!("保存失败，但目前不做处理: {:?}", error);
                }
                iced::window::latest().and_then(iced::window::close)
            }
            Message::Reset => {
                *self = Self::default();
                Task::none()
            }
            Message::DetailMessage(i, detail_message) => {
                if let Some(detail) = self.details.get_mut(i) {
                    detail
                        .update(detail_message)
                        .map(Message::DetailMessage.with(i))
                } else {
                    Task::none()
                }
            }
            Message::RunMessage(run_message) => self
                .runner
                .update(run_message, self.get_details())
                .map(Message::RunMessage),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = text!("物流对账系统")
            .width(Fill)
            .align_x(Center)
            .size(H1_SIZE);
        let tail = row![
            space::horizontal(),
            button("重置")
                .style(button::secondary)
                .on_press(Message::Reset),
            button("运行").on_press(Message::RunMessage(RunMessage::Run)),
            space().width(5. * SPACING)
        ]
        .spacing(2. * SPACING)
        .width(Fill);
        let details: Element<_> = row(self
            .details
            .iter()
            .enumerate()
            .map(|(i, detail)| detail.view().map(Message::DetailMessage.with(i))))
        .spacing(CONTAINER_SPACING)
        .height(Fill)
        .into();

        let content: Element<_> = container(
            column![title, details, tail]
                .spacing(SPACING * 2.)
                .align_x(Center),
        )
        .center_x(Fill)
        .padding(CONTAINER_PADDING)
        .into();

        // 展示弹窗, 矛盾点是有多个 headers 需要遍历来展示
        let template = self
            .details
            .iter()
            .enumerate()
            .flat_map(|(i, detail)| {
                detail
                    .templates
                    .iter()
                    .enumerate()
                    .filter_map(move |(j, template)| {
                        if template.show_header_view {
                            Some((i, j, template))
                        } else {
                            None
                        }
                    })
            })
            .nth(0);
        // 条件渲染表头模板
        if let Some((i, j, template)) = template {
            modal(
                content,
                template.header_view().map(move |template_message| {
                    Message::DetailMessage(
                        i,
                        DetailMessage::TempMessage(j, Box::new(template_message)),
                    )
                }),
                Message::DetailMessage(
                    i,
                    DetailMessage::TempMessage(j, Box::new(TemplateMessage::HideHeaderView)),
                ),
            )
        } else if self.runner.show_result && self.runner.result.is_some() {
            modal(
                content,
                self.runner.view().map(Message::RunMessage),
                Message::RunMessage(RunMessage::HideResult),
            )
        } else {
            content
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            details: [
                BillDetails {
                    title: "物流账单".into(),
                    avaliable_templates: vec![
                        ParserType::WB,
                        ParserType::TS,
                        ParserType::DDD,
                        ParserType::GRT,
                    ]
                    .into(),
                    current_template: None,
                    templates: vec![],
                },
                BillDetails {
                    title: "我方明细".into(),
                    avaliable_templates: vec![ParserType::Headway].into(),
                    current_template: None,
                    templates: vec![],
                },
            ],
            runner: Runner::default(),
            exiting: false,
        }
    }
}

impl State {
    pub fn get_details(&self) -> [Vec<UserData>; 2] {
        [
            self.details[0].get_user_data(),
            self.details[1].get_user_data(),
        ]
    }
}

impl BillDetails {
    pub fn get_user_data(&self) -> Vec<UserData> {
        self.templates
            .iter()
            .filter_map(|t| {
                t.get_sheets().map(|sheets| UserData {
                    parser_type: t.parser_type.clone(),
                    headers: t.headers.clone(),
                    sheets,
                    primary: "序号".into(),
                })
            })
            .collect()
    }
}

impl Template {
    pub fn get_sheets(&self) -> Option<Vec<(PathBuf, String)>> {
        let sheets: Vec<(PathBuf, String)> = self
            .sheets
            .iter()
            .filter_map(|t| {
                if t.checked
                    && let Some(name) = t.select_sheet.as_ref()
                {
                    Some((t.path.clone(), name.to_owned()))
                } else {
                    None
                }
            })
            .collect();
        if !sheets.is_empty() {
            Some(sheets)
        } else {
            None
        }
    }
}

impl SaveState {
    fn path() -> std::path::PathBuf {
        let mut path =
            if let Some(project_dirs) = directories::ProjectDirs::from("rs", "Iced", "Logirecon") {
                project_dirs.data_dir().into()
            } else {
                std::env::current_dir().unwrap_or_default()
            };

        path.push("state.json");

        path
    }

    async fn load() -> Result<Self, LoadError> {
        let contents = tokio::fs::read_to_string(Self::path())
            .await
            .map_err(|_| LoadError::File)?;

        serde_json::from_str(&contents).map_err(|_| LoadError::Format)
    }

    async fn save(self) -> Result<(), SaveError> {
        // use iced::time::milliseconds;

        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::Format)?;

        let path = Self::path();

        if let Some(dir) = path.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|_| SaveError::Write)?;
        }

        {
            tokio::fs::write(path, json.as_bytes())
                .await
                .map_err(|_| SaveError::Write)?;
        }

        // This is a simple way to save at most twice every second
        // tokio::time::sleep(milliseconds(500)).await;

        Ok(())
    }
}
