use dialoguer::theme::ColorfulTheme;

use crate::models::Input;

pub fn get_user_input() -> anyhow::Result<Input> {
    let title: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Masukan judul")
        .interact()?;

    let tipe = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Tipe pencarian")
        .default(0)
        .item("Anime on-going dan complete")
        .item("Film asia dan anime live action")
        .item("Film Box Office dan TV Series")
        .interact()?;

    Ok(Input { title, tipe })
}

pub fn choice<T: std::fmt::Display + Clone>(options: Vec<T>, fuzzy: bool) -> anyhow::Result<T> {
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
