rust_i18n::i18n!("locales", fallback = "en");

mod config;
mod detection;
mod drivers;
mod executor;
mod flash;
mod images;
mod jtag;
mod ui;

use config::AppConfig;

fn main() {
    // Load saved config (or defaults)
    let mut config = AppConfig::load();

    // Set locale from config
    rust_i18n::set_locale(&config.locale);

    // Ensure required directories exist
    config.ensure_dirs();

    // Enter main menu loop
    ui::menus::main_menu(&mut config);
}
