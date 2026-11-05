use anyhow::Result;
use dialoguer::{Confirm, FuzzySelect, Input, Select, theme::ColorfulTheme};

pub fn theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

/// Short fixed-list Select (arrow keys + enter)
pub fn select(prompt: &str, items: &[&str]) -> Result<usize> {
    let idx = Select::with_theme(&theme())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()?;
    Ok(idx)
}

/// Select with String slice
pub fn select_str(prompt: &str, items: &[String]) -> Result<usize> {
    let refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    select(prompt, &refs)
}

/// FuzzySelect for longer lists (type to filter, arrow keys to navigate)
pub fn fuzzy_select(prompt: &str, items: &[&str]) -> Result<usize> {
    let idx = FuzzySelect::with_theme(&theme())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()?;
    Ok(idx)
}

pub fn fuzzy_select_str(prompt: &str, items: &[String]) -> Result<usize> {
    let refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    fuzzy_select(prompt, &refs)
}

/// Yes/No confirmation
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    let ans = Confirm::with_theme(&theme())
        .with_prompt(prompt)
        .default(default)
        .interact()?;
    Ok(ans)
}

/// Free text input with optional default
pub fn input(prompt: &str, default: Option<&str>) -> Result<String> {
    let theme = theme();
    let inp: Input<String> = Input::with_theme(&theme);
    let inp = inp.with_prompt(prompt);
    let inp = if let Some(d) = default {
        inp.default(d.to_string())
    } else {
        inp
    };
    Ok(inp.interact_text()?)
}
