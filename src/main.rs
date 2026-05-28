use logirecon::{Result, app::EguiApp};

fn main() -> Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        EguiApp::APP_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(EguiApp::new(cc)))),
    )?;
    Ok(())
}
