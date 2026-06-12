#![cfg(feature = "gui")]
#![windows_subsystem = "windows"]
use logirecon_gui::application;

fn main() -> iced::Result {
    application().run()
}
