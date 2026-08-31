#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() -> iced::Result {
    let file_appender = {
        #[cfg(not(debug_assertions))]
        let data_dir =
            if let Some(project_dir) = directories::ProjectDirs::from("rs", "Iced", "Logirecon") {
                project_dir.cache_dir().into()
            } else {
                std::env::current_dir().unwrap_or_default()
            };
        #[cfg(debug_assertions)]
        let data_dir = "./";
        tracing_appender::rolling::daily(data_dir, "logirecon.log")
    };
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_env_filter("logirecon=info,logirecon_iced=info,iced_wpgu=warn")
        .with_ansi(false)
        .with_writer(non_blocking)
        .init();
    logirecon_iced::app::application().run()
}
