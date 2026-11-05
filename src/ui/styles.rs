use console::style;
use rust_i18n::t;

pub fn print_header() {
    println!();
    println!(
        "{}",
        style("╔══════════════════════════════════════════════════════╗").cyan()
    );
    println!(
        "{}  {}  {}",
        style("║").cyan(),
        style(format!("{:<52}", format!("  {}  —  {}", t!("app.name"), t!("app.tagline")))).bold().white(),
        style("║").cyan()
    );
    println!(
        "{}",
        style("╚══════════════════════════════════════════════════════╝").cyan()
    );
    println!();
}

#[allow(dead_code)]
pub fn print_header_simple() {
    println!();
    println!(
        "{}  {}",
        style("▶").cyan().bold(),
        style(format!("{} — {}", t!("app.name"), t!("app.tagline")))
            .bold()
            .white()
    );
    println!("{}", style("─".repeat(60)).dim());
    println!();
}

pub fn print_section(title: &str) {
    println!();
    println!("{}", style(format!("── {} ", title)).cyan().bold());
    println!();
}

pub fn ok(msg: &str) {
    println!("  {}  {}", style("✔").green().bold(), style(msg).green());
}

pub fn err(msg: &str) {
    println!("  {}  {}", style("✘").red().bold(), style(msg).red());
}

pub fn warn(msg: &str) {
    println!("  {}  {}", style("⚠").yellow().bold(), style(msg).yellow());
}

pub fn info(msg: &str) {
    println!("  {}  {}", style("ℹ").blue().bold(), style(msg));
}

pub fn step(msg: &str) {
    println!("  {}  {}", style("›").cyan().bold(), msg);
}

pub fn dim(msg: &str) {
    println!("  {}", style(msg).dim());
}

pub fn press_enter() {
    println!();
    print!("  {} ", style(t!("common.press_enter")).dim());
    let _ = std::io::stdin().read_line(&mut String::new());
}
