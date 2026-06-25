#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() -> iced::Result {
    logirecon_iced::app::application().run()
}
