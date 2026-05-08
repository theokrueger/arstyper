//! arstyper
mod config;
mod lang;
mod traits;
mod ui;

use config::Config;
use ui::Ui;

macro_rules! err_disp {
    ($name:literal) =>  {
        |e| {
            println!("Fatal Error in {}: {}" ,$name, e);
            std::process::exit(1);
        }
    }
}

fn main() -> std::io::Result<()> {
    let cfg = Config::get().unwrap_or_else(err_disp!("Config"));
    let ui = Ui::new(cfg).unwrap_or_else(err_disp!("UI"));
    ui.run().unwrap_or_else(err_disp!("UI"));
    Ok(())
}
