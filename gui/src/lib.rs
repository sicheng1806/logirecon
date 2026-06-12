mod app;
mod detail;
mod runner;
mod sheet;
mod template;

pub use app::{App, AppMessage, Message, State, application};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use iced::{
    Element, Font, Theme, font,
    widget::{button, center, container, mouse_area, opaque, stack, svg, table, text},
};

const SPACING: f32 = 10.;
const CONTAINER_PADDING: f32 = 20.;
const CONTAINER_SPACING: f32 = 20.;
const EXCEL_SUFFIX: [&str; 6] = ["xls", "xlm", "xlsx", "xlsm", "xlsb", "ods"];
const H1_SIZE: u32 = 24;
const H2_SIZE: u32 = 20;

#[derive(Debug)]
pub struct UserData {
    pub parser_type: ParserType,
    pub headers: HashMap<String, String>,
    pub sheets: Vec<(PathBuf, String)>,
    pub primary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParserType {
    WB,
    TS,
    GRT,
    DDD,
    Headway,
}

impl std::fmt::Display for ParserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl ParserType {
    pub fn name(&self) -> &str {
        match self {
            Self::Headway => "头程明细",
            Self::WB => "万邦",
            Self::TS => "天盛",
            Self::DDD => "嘀嗒嘀",
            Self::GRT => "国润通",
        }
    }
}

fn svg_button<'a, Message>(path: &str) -> button::Button<'a, Message, iced::Theme, iced::Renderer> {
    let svg = svg(format!("{}{}", env!("CARGO_MANIFEST_DIR"), path))
        .width(20)
        .height(20)
        .style(|theme: &Theme, _state| {
            let palette = theme.extended_palette();
            svg::Style {
                color: Some(palette.background.base.text),
            }
        });
    button(svg).style(button::subtle)
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(mouse_area(center(opaque(content)).style(container::rounded_box)).on_press(on_blur))
    ]
    .into()
}

fn dataframe_table<'a, Message>(df: &'a polars::frame::DataFrame) -> table::Table<'a, Message>
where
    Message: Clone + 'a,
{
    use polars::prelude::AnyValue;
    let bold = |header| {
        text(header).font(Font {
            weight: font::Weight::Bold,
            ..Font::DEFAULT
        })
    };
    let columns: Vec<table::Column<_, _>> = df
        .schema()
        .iter_names()
        .enumerate()
        .map(|(i, t)| {
            table::column(bold(t.to_string()), move |row: Vec<AnyValue<'_>>| {
                text(format!("{}", row[i]))
            })
        })
        .collect();
    let mut rows = vec![];
    for i in 0..df.height() {
        rows.push(df.get_row(i).unwrap().0);
    }
    table(columns, rows)
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_use_rfd() {
        use rfd::FileDialog;
        let files = FileDialog::new()
            .add_filter("excel", &["xlsx", "xlsm", "xls"])
            .set_directory("~")
            .pick_file();
        println!("{:?}", files);
    }
}
