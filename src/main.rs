use std::io::{stdout, Write};
use std::process::Command;

use animeku::AnimekuCli;
use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use ext::{nontonanime, Ext};
use tokio::runtime;

use crate::{ext::tenflix, input::get_user_input, util::clearscreen_and_show_banner};

mod animeku;
mod ext;
mod input;
mod models;
mod util;

fn get_ext(id: usize) -> Box<dyn Ext> {
    if id == 0 {
        Box::new(nontonanime::anime::AnimeExt::new())
    } else if id == 1 {
        Box::new(nontonanime::movie::MovieExt::new())
    } else {
        Box::new(tenflix::TenflixExt::new())
    }
}

#[allow(unreachable_code)]
async fn app() -> anyhow::Result<()> {
    clearscreen_and_show_banner()?;

    let input = get_user_input()?;
    let extractor = get_ext(input.tipe);

    let mut animeku = AnimekuCli::new(extractor);

    clearscreen_and_show_banner()?;
    let movie = animeku.search(&input.title).await?;

    loop {
        clearscreen_and_show_banner()?;
        let episode = animeku.extract_episode(movie.clone()).await?;
        clearscreen_and_show_banner()?;
        let is_series = episode.is_series;
        let download = animeku.extract_stream_urls(episode).await?;

        print!("{} Membuka tautan diaplikasi eksternal .. {}", "◆".blue(), "\n");
        stdout().flush()?;

        if cfg!(target_os = "linux") {
            if dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Apakah kamu ingin membukanya di MPV (Jika Tersedia)")
            .interact()?
            {
                let status = Command::new("mpv")
                .arg(download.url)
                .status()
                .expect("Failed to start 'mpv'");

                if !status.success() {
                    eprintln!(
                        "{} Gagal menjalankan MPV",
                        "■".red()
                    );
                    break;
                }
            } else {
                if open::that(download.url).is_ok() {
                    if !is_series {
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
        }
    }
    println!();
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let rt = runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async {
        if let Err(e) = app().await {
            println!(" {} {}\n", "■".red(), format!("{:#?}", e).yellow());
        }
    });
    Ok(())
}
