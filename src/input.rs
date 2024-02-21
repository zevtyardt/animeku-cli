use colored::Colorize;
use dialoguer::theme::ColorfulTheme;

use crate::models::Input;

pub fn get_user_input() -> anyhow::Result<Input> {
    let title: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Masukan judul")
        .interact()?;

    let tipe = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Tipe pencarian")
        .default(0)
        .item("Anime: on-going dan complete")
        .item("Movie: live action, film asia dan film anime")
        .interact()?;

    Ok(Input { title, tipe })
}

pub fn choice<T: std::fmt::Display + Clone>(options: Vec<T>, fuzzy: bool) -> anyhow::Result<T> {
    if options
        .clone()
        .iter()
        .filter(|v| !v.to_string().contains("Halaman"))
        .count()
        == 1
    {
        let selected = options[0].clone();
        println!("{} Pilih · {}", "✔".green(), selected.to_string().green());
        return Ok(selected);
    }
    let selected = if !fuzzy {
        dialoguer::Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pilih")
            .default(0)
            .max_length(5)
            .items(&options)
            .interact()?
    } else {
        dialoguer::FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Pilih")
            .default(0)
            .max_length(5)
            .items(&options)
            .interact()?
    };
    Ok(options[selected].clone())
}
