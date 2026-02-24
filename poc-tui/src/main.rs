use crate::app::App;

pub mod app;
pub mod debugger_ctx;
pub mod event;
pub mod function_mapping;
pub mod ui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
