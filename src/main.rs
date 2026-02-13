//! arstyper
mod config;
mod lang;
mod ui;

use config::Config;
use ui::Ui;

fn main() -> std::io::Result<()> {
    let cfg = Config::get()?;
    let mut ui = Ui::new(cfg);
    ui.run()?;
    Ok(())
}
