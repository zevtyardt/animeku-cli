use std::{
    collections::HashMap,
    io::{stdout, Write},
    time::Duration,
};

use colored::Colorize;
use reqwest::Client;
use serde_json::Value;

use crate::{
    ext::Ext,
    input,
    models::{Download, Episode, Movie},
    util::{get_filesize, get_real_url},
};

static USER_PASS: &str = "drakornicojanuar:DIvANTArtBInsTriSkEremeNtOMICErCeSMiQUaKarypsBoari";

pub struct AnimekuCli {
    cache: HashMap<usize, Vec<Movie>>,
    extractor: Box<dyn Ext>,
}

impl AnimekuCli {
    pub fn new(extractor: Box<dyn Ext>) -> Self {
        Self {
            cache: HashMap::new(),
            extractor,
        }
    }

    pub async fn search(&mut self, search_title: &str) -> anyhow::Result<Movie> {
        print!("{} Proses pencarian .. ", "◆".blue());
        stdout().flush()?;

        let mut page = 1;
        let mut is_latest = false;

        loop {
            if let std::collections::hash_map::Entry::Vacant(_) = self.cache.entry(page) {
                let (movie_list, total) = self.extractor.search(search_title.into(), page).await?;
                if page == 1 && !movie_list.is_empty() {
                    println!("ditemukan {} judul anime", total.to_string().green());
                }
                self.cache.insert(page, movie_list);
            }

            let mut movie_list = self.cache.get(&page).unwrap().clone();
            if movie_list.is_empty() {
                if page == 1 {
                    eprintln!(
                        "tidak ditemukan!\n{} {}",
                        "■".red(),
                        "Silahkan periksa kembali kata kunci pencarian\n".yellow()
                    );
                    std::process::exit(0);
                }
                page -= 1;
                is_latest = true;
                continue;
            }

            if page > 1 {
                movie_list.push(Movie {
                    channel_id: 1,
                    title: format!("Sebelumnnya (Halaman {})", page - 1),
                    total_episodes: None,
                })
            }

            if !is_latest {
                movie_list.push(Movie {
                    channel_id: 2,
                    title: format!("Selanjutnya (Halaman {})", page + 1),
                    total_episodes: None,
                })
            }

            let movie = input::choice(movie_list, false)?;
            if movie.channel_id == 1 {
                page -= 1;
            } else if movie.channel_id == 2 {
                page += 1;
            } else {
                return Ok(movie);
            }
        }
    }

    #[allow(unused_must_use)]
    pub async fn extract_episode(&self, movie: Movie) -> anyhow::Result<Episode> {
        print!("{} Mengambil semua daftar episode .. ", "◆".blue());
        stdout().flush()?;

        let (episodes, meta) = self.extractor.get_episodes(movie).await?;

        if episodes.is_empty() {
            println!("tidak berhasil!");
            std::process::exit(0);
        }

        println!("berhasil");

        println!(
            "\n  {}{}\n",
            " ".repeat(45 / 2 - 5),
            " Deskripsi ".black().on_truecolor(252, 136, 3)
        );

        if let Some(thumb_url) = meta.thumb_url {
            self.show_image_thumb(thumb_url).await;
        }

        for (k, v) in meta.data {
            println!("  {:<9}: {}", k.bright_white(), v);
        }

        println!("  {:<9}: {}", "Episode".bright_white(), episodes.len());
        println!();

        let selected = input::choice(episodes, true)?;
        Ok(selected)
    }

    pub async fn extract_download_url(&self, episode: Episode) -> anyhow::Result<Download> {
        print!("{} Memuat tautan unduhan .. ", "◆".green());
        stdout().flush()?;

        let url =
            "https://animeku.my.id/nontonanime-v77/phalcon/api/get_post_description_secure/v9_4/";
        let payload = format!("channel_id={}&isAPKvalid=true", episode.channel_id);

        let client = Client::new();
        let response = client
            .post(url)
            .header("Cache-Control", "max-age=0")
            .header("Data-Agent", "New Aniplex v9.1")
            .header("Accept-Encoding", "gzip")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Content-Length", payload.len())
            .header("Host", "animeku.my.id")
            .header("Connection", "Keep-Alive")
            .header("User-Agent", "okhttp/3.12.13")
            .body(payload)
            .send()
            .await?;

        let json = response.json::<Value>().await?;
        let mut urls = vec![];
        for (n, reso) in [
            ("channel_url", "360p SD"),
            ("channel_url_hd", "720p HD"),
            ("channel_url_fhd", "1080p FHD"),
        ] {
            if let Some(raw_url) = json[n].as_str() {
                let mut reso = reso.to_string();
                let mut url = raw_url.trim_matches('"').trim().to_string();
                if url.contains("whatbox") {
                    if let Some(new_url) = url.strip_prefix("http://") {
                        url = format!("http://{}@{}", USER_PASS, new_url);
                    }
                } else if url.contains("nontonanime") {
                    if let Ok(direct_url) = get_real_url(&client, url.clone()).await {
                        url = direct_url
                    }
                }

                if url.starts_with("http") {
                    if let Some(size) = get_filesize(&client, &url).await {
                        reso.push_str(" (");
                        reso.push_str(size.as_str());
                        reso.push(')')
                    }

                    urls.push(Download { url, title: reso });
                }
            }
        }

        if urls.is_empty() {
            eprintln!(
                "gagal!\n{} {}",
                "■".red(),
                "Silahkan jalankan ulang program\n".yellow()
            );
            std::process::exit(0);
        }
        println!("berhasil");

        let selected = input::choice(urls, false)?;
        Ok(selected)
    }

    pub async fn show_image_thumb(&self, url: String) -> anyhow::Result<()> {
        let client = Client::new();
        let resp = client
            .get(url)
            .timeout(Duration::from_secs(2))
            .send()
            .await?;
        let bytes = resp.bytes().await?;

        let img = image::load_from_memory(&bytes)?;
        let conf = viuer::Config {
            transparent: true,
            width: Some(50),
            height: Some(30),
            y: 12,
            x: 2,
            ..Default::default()
        };
        viuer::print(&img, &conf)?;
        println!();
        Ok(())
    }
}
