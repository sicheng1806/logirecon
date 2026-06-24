//! 用于在Excel工作表中选择表名

use std::path::PathBuf;

use super::components::trash_button;
use iced::{Element, widget::combo_box};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub id: uuid::Uuid,
    pub path: PathBuf,
    pub selected: Option<String>,
    pub chosen: bool,
    pub sheets: ComboState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Delete,
    Chosen(bool),
    Selected(String),
}

impl State {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, calamine::Error> {
        use calamine::{Reader, open_workbook_auto};
        let path = path.into();
        let wb = open_workbook_auto(&path)?;
        let sheets = wb.sheet_names();
        Ok(Self {
            id: uuid::Uuid::new_v4(),
            path,
            selected: None,
            chosen: false,
            sheets: sheets.into(),
        })
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Delete => {}
            Message::Chosen(chosen) => self.chosen = chosen,
            Message::Selected(name) => self.selected = Some(name),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::{Alignment::*, widget::*};
        let check_btn = checkbox(self.chosen)
            .label(self.path.file_name().unwrap().to_str().unwrap())
            .on_toggle(Message::Chosen);
        let select = combo_box(
            &self.sheets.0,
            "选择工作簿",
            self.selected.as_ref(),
            Message::Selected,
        );
        let delete_btn = trash_button().on_press(Message::Delete);
        row![check_btn, select, delete_btn]
            .spacing(10)
            .align_y(Center)
            .into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "Vec<String>", into = "Vec<String>")]
pub struct ComboState(combo_box::State<String>);

impl From<Vec<String>> for ComboState {
    fn from(value: Vec<String>) -> Self {
        Self(combo_box::State::new(value))
    }
}

impl From<ComboState> for Vec<String> {
    fn from(value: ComboState) -> Self {
        value.0.into_options()
    }
}
