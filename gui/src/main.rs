use logirecon_gui::application;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    application().run()
}
