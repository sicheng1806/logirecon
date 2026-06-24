mod app;
mod components;
mod constants;
mod detail;
mod modal;
mod runner;
mod sheet;
mod template;
mod window;

fn main() -> iced::Result {
    app::application().run()
}
