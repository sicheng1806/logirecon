use super::constants::*;
use iced::{
    Element, Font, Renderer, Theme, font,
    widget::{button, center, container, mouse_area, opaque, stack, svg, table, text},
};

pub fn trash_button<'a, Message>() -> button::Button<'a, Message, Theme, Renderer> {
    svg_button("/public/trash.svg")
}

pub fn cancel_button<'a, Message>() -> button::Button<'a, Message, Theme, Renderer> {
    svg_button("/public/cancel.svg")
}

pub fn svg_button<'a, Message>(
    path: &str,
) -> button::Button<'a, Message, iced::Theme, iced::Renderer> {
    let svg = svg(format!("{}{}", env!("CARGO_MANIFEST_DIR"), path))
        .width(PADDING)
        .height(PADDING)
        .style(|theme: &Theme, _state| {
            let palette = theme.extended_palette();
            svg::Style {
                color: Some(palette.background.base.text),
            }
        });
    button(svg).style(button::subtle)
}

pub fn modal<'a, Message>(
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

pub fn dataframe_table<'a, Message>(df: &'a polars::frame::DataFrame) -> table::Table<'a, Message>
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
