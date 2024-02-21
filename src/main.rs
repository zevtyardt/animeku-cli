#![allow(dead_code)]

use std::io::{stdout, Write};

use animeku::AnimekuCli;
use colored::Colorize;
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
    loop {
        init_app()?;

        let input = get_user_input()?;

        let extractor: Box<dyn Ext> = if input.tipe == 0 {
            AnimeExt::new()
        } else {
            MovieExt::new()
        };

        let mut animeku = AnimekuCli::new(extractor);
        let movie = animeku.search(&input.title).await?;

        let episode = animeku.extract_episode(movie).await?;
        let download = animeku.extract_download_url(episode).await?;

        print!("{} Membuka tautan diaplikasi eksternal .. ", "◆".blue());
        stdout().flush()?;

        let status = if open::that(download.url).is_ok() {
            "berhasil"
        } else {
            "gagal"
        };
        println!("{}\n", status);

        if dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("apakah kamu ingin keluar")
            .interact()?
        {
            break;
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let rt = runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(app())
}
