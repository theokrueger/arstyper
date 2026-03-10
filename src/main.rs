//! arstyper
mod config;
mod lang;
mod test;
mod ui;
mod color_preview;

use config::Config;
use ui::Ui;

fn main() -> std::io::Result<()> {
    let cfg = Config::get()?;
    let ui = Ui::new(cfg)?;
    ui.run()?;
    Ok(())
}
