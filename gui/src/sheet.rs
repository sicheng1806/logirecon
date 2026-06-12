use super::svg_button;
use calamine::open_workbook_auto;
use iced::{
    Alignment::*,
    Element,
    widget::{checkbox, combo_box, row},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 内部Sheet状态，实现序列化和反序列化
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "Vec<String>", into = "Vec<String>")]
pub struct SheetState(combo_box::State<String>);

impl From<Vec<String>> for SheetState {
    fn from(value: Vec<String>) -> Self {
        Self(combo_box::State::new(value))
    }
}

impl From<SheetState> for Vec<String> {
    fn from(value: SheetState) -> Self {
        value.0.into_options()
    }
}

/// Sheet 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub id: uuid::Uuid,
    pub path: PathBuf,
    pub select_sheet: Option<String>,
    pub checked: bool,
    pub sheets: SheetState,
}

#[derive(Debug, Clone)]
pub enum SheetMessage {
    /// 外部信息
    Delete,
    Checked(bool),
    Selected(String),
}

impl Sheet {
    pub fn new_from_path(path: impl Into<PathBuf>) -> Option<Self> {
        use calamine::Reader;
        let path = path.into();
        if let Ok(wb) = open_workbook_auto(&path) {
            let sheets = wb.sheet_names();
            Some(Self {
                id: uuid::Uuid::new_v4(),
                path,
                select_sheet: None,
                checked: false,
                sheets: sheets.into(),
            })
        } else {
            None
        }
    }

    pub fn update(&mut self, message: SheetMessage) {
        match message {
            SheetMessage::Checked(new_choosed) => {
                self.checked = new_choosed;
            }
            SheetMessage::Selected(name) => {
                self.select_sheet = Some(name);
            }
            SheetMessage::Delete => {}
        }
    }

    pub fn view(&self) -> Element<'_, SheetMessage> {
        let check_btn = checkbox(self.checked)
            .label(self.path.file_name().unwrap().to_str().unwrap())
            .on_toggle(SheetMessage::Checked);
        let select = combo_box(
            &self.sheets.0,
            "选择工作簿",
            self.select_sheet.as_ref(),
            SheetMessage::Selected,
        );
        let delete_btn = svg_button("/public/trash.svg").on_press(SheetMessage::Delete);
        row![check_btn, select, delete_btn]
            .spacing(10)
            .align_y(Center)
            .into()
    }
}
