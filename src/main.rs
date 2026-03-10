//! arstyper
mod color_preview;
mod config;
mod lang;
mod results;
mod test;
mod ui;

use config::Config;
use ui::Ui;

fn main() -> std::io::Result<()> {
    let cfg = Config::get()?;
    let ui = Ui::new(cfg)?;
    ui.run()?;
    Ok(())
}
