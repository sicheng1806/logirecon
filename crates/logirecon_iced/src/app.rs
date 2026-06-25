//! GUI的应用程序
//!
//! 在 [window] 的基础上添加了载入功能和关闭窗口自动保存功能
use iced::{Element, Program, Task, Theme};
use serde::{Deserialize, Serialize};

use crate::detail;

use super::window;

pub fn application() -> iced::Application<impl Program<Message = Message, Theme = Theme>> {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .exit_on_close_request(false)
}

pub enum App {
    Loading,
    Loaded(Box<window::State>),
}

pub enum Message {
    Loaded(Box<Result<SaveState, LoadError>>),
    WindowMessage(window::Message),
}

impl Default for App {
    fn default() -> Self {
        Self::Loaded(Box::default())
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self::Loading,
            Task::perform(SaveState::load(), |res| Message::Loaded(Box::new(res))),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match self {
            Self::Loading => {
                if let Message::Loaded(res) = message {
                    *self = if let Ok(state) = *res {
                        Self::Loaded(Box::new(window::State {
                            details: state.details,
                            ..Default::default()
                        }))
                    } else {
                        Self::Loaded(Box::default())
                    }
                };
                Task::none()
            }
            Self::Loaded(state) => {
                if let Message::WindowMessage(message) = message {
                    match message {
                        window::Message::Save => {
                            let save_state = SaveState {
                                details: state.details.clone(),
                            };
                            Task::perform(save_state.save(), |_res| window::Message::Exit)
                                .map(Message::WindowMessage)
                        }
                        _ => state.update(message).map(Message::WindowMessage),
                    }
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::text;
        match self {
            Self::Loading => text!("加载中...").into(),
            Self::Loaded(state) => state.view().map(Message::WindowMessage),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::window::close_requests()
            .filter_map(|_window_id| Some(Message::WindowMessage(window::Message::Save)))
    }
}

pub enum LoadError {
    File,
    Format,
}

pub enum SaveError {
    Write,
    Format,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveState {
    details: [detail::State; 2],
}

impl SaveState {
    fn path() -> std::path::PathBuf {
        let mut path =
            if let Some(project_dir) = directories::ProjectDirs::from("rs", "Iced", "Logirecon") {
                project_dir.data_dir().into()
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
