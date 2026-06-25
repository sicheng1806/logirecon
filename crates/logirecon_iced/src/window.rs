//! 主窗口组件
//!
//! 包含由[detail]、[modal]、[runner]组成的窗口组件

use super::{
    components::modal,
    constants::{H1_SIZE, PADDING, SPACING},
    modal::ModalView,
};

use super::{detail, runner, template};
use iced::{Element, Function, Task};

#[derive(Debug, Clone)]
pub struct State {
    pub details: [detail::State; 2],
    pub modal_state: Option<ModalState>,
    pub runner: runner::State,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Detail(usize, detail::Message),
    Runner(runner::Message),
    ShowModal(Option<ModalState>),
    Reset,
    Save,
    Exit,
}

impl State {
    pub fn new() -> Self {
        Self {
            details: [detail::State::bill(), detail::State::shipment()],
            modal_state: None,
            runner: runner::State::default(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Save => Task::none(),
            Message::Reset => {
                *self = Self::default();
                Task::none()
            }
            Message::Exit => iced::window::latest().and_then(iced::window::close),
            Message::ShowModal(state) => {
                self.modal_state = state;
                Task::none()
            }
            Message::Detail(i, message) => {
                if let detail::Message::Template(j, template_message) = &message
                    && let template::Message::ShowModal(showing) = **template_message
                {
                    if showing {
                        Task::done(Message::ShowModal(Some(ModalState::Template(i, *j))))
                    } else {
                        Task::done(Message::ShowModal(None))
                    }
                } else {
                    if let Some(state) = self.details.get_mut(i) {
                        state.update(message).map(Message::Detail.with(i))
                    } else {
                        Task::none()
                    }
                }
            }
            Message::Runner(runner::Message::ShowModal(showing)) => {
                if showing {
                    Task::done(Message::ShowModal(Some(ModalState::Runner)))
                } else {
                    Task::done(Message::ShowModal(None))
                }
            }
            Message::Runner(message) => self.runner.update(message).map(Message::Runner),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::{
            Alignment::*,
            Length::*,
            widget::{button, column, container, row, space, text},
        };
        let title = text!("物流对账系统")
            .width(Fill)
            .align_x(Center)
            .size(H1_SIZE);
        let footer = row![
            space::horizontal(),
            button("重置")
                .style(button::secondary)
                .on_press(Message::Reset),
            button("运行").on_press(Message::Runner(runner::Message::Run(
                self.details.clone().map(|t| t.templates)
            ))),
            space().width(5. * SPACING)
        ]
        .spacing(2. * SPACING)
        .width(Fill);
        let body = row(self
            .details
            .iter()
            .enumerate()
            .map(|(i, state)| state.view().map(Message::Detail.with(i))))
        .spacing(SPACING)
        .height(Fill);
        let base = container(
            column![title, body, footer]
                .spacing(SPACING * 2.)
                .align_x(Center),
        )
        .center_x(Fill)
        .padding(PADDING)
        .into();
        if self.modal_state.is_none() {
            base
        } else {
            modal(base, self.modal_view(), Message::ShowModal(None))
        }
    }
}

impl ModalView for State {
    type Message = Message;

    fn modal_view(&self) -> Element<'_, Self::Message> {
        use iced::widget::text;
        if let Some(state) = self.modal_state.as_ref() {
            match state {
                ModalState::Template(i, j) => {
                    if let Some(detail) = self.details.get(*i)
                        && let Some(temp) = detail.templates.get(*j)
                    {
                        temp.modal_view().map(move |m| {
                            Message::Detail(*i, detail::Message::Template(*j, Box::new(m)))
                        })
                    } else {
                        text!("实现错误，请检查").center().into()
                    }
                }
                ModalState::Runner => self.runner.modal_view().map(Message::Runner),
            }
        } else {
            text!("实现错误，请检查").center().into()
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModalState {
    Template(usize, usize),
    Runner,
}
