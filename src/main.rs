use std::io::{stdout, Write};

use animeku::AnimekuCli;
use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use ext::{anime::AnimeExt, movie::MovieExt, Ext};
use tokio::runtime;

use crate::input::get_user_input;

mod animeku;
mod ext;
mod input;
mod models;
mod util;

fn init_app() -> anyhow::Result<()> {
    clearscreen::clear()?;

    eprintln!(
        "{} v{} {} val \n",
        r#"
  ▄▀█ █▄░█ █ █▀▄▀█ █▀▀ █▄▀ █░█ ▄▄ █▀▀ █░░ █
  █▀█ █░▀█ █ █░▀░█ ██▄ █░█ █▄█ ░░ █▄▄ █▄▄ █ "#
            .bright_green(),
        env!("CARGO_PKG_VERSION"),
        "©".cyan()
    );
    Ok(())
}

async fn app() -> anyhow::Result<()> {
    init_app()?;

    let input = get_user_input()?;
    let extractor: Box<dyn Ext> = if input.tipe == 0 {
        AnimeExt::new()
    } else {
        MovieExt::new()
    };

    let mut animeku = AnimekuCli::new(extractor);

    init_app()?;
    let movie = animeku.search(&input.title).await?;

    loop {
        init_app()?;
        let episode = animeku.extract_episode(movie.clone()).await?;
        init_app()?;
        let is_anime = episode.is_anime;
        let download = animeku.extract_download_url(episode).await?;

        print!("{} Membuka tautan diaplikasi eksternal .. ", "◆".blue());
        stdout().flush()?;

        if open::that(download.url).is_ok() {
            println!("berhasil");

            if !is_anime {
                break;
            }
            if dialoguer::Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Apakah kamu ingin keluar")
                .interact()?
            {
                break;
            }
        } else {
            println!("gagal");
            break;
        };
    }
    println!();
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let rt = runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(app())
}
