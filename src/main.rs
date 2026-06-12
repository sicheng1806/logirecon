#![cfg(feature = "gui")]
use logirecon_gui::application;

fn main() -> iced::Result {
    application().run()
}
