use std::{
    collections::HashMap,
    io::{stdout, Write},
};

use colored::Colorize;

use crate::{
    ext::Ext,
    input,
    models::{Episode, Meta, Movie, Stream},
    util::show_image_thumb,
};

pub struct AnimekuCli {
    movie_cache: HashMap<String, Vec<Movie>>,
    episode_cache: HashMap<String, (Vec<Episode>, Meta)>,
    stream_cache: HashMap<String, Vec<Stream>>,
    extractor: Box<dyn Ext>,
}

impl AnimekuCli {
    pub fn new(extractor: Box<dyn Ext>) -> Self {
        Self {
            movie_cache: HashMap::new(),
            episode_cache: HashMap::new(),
            stream_cache: HashMap::new(),
            extractor,
        }
    }

    pub async fn search(&mut self, search_title: &str) -> anyhow::Result<Movie> {
        print!(
            "{} Proses pencarian '{}'.. ",
            "◆".blue(),
            search_title.green()
        );
        stdout().flush()?;

        let mut page = 1;
        let mut is_latest = false;

        loop {
            if let std::collections::hash_map::Entry::Vacant(_) =
                self.movie_cache.entry(page.to_string())
            {
                let (movie_list, total) = self.extractor.search(search_title.into(), page).await?;
                if page == 1 && !movie_list.is_empty() {
                    println!("ditemukan {} judul", total.to_string().green());
                }
                self.movie_cache.insert(page.to_string(), movie_list);
            }

            let mut movie_list = self.movie_cache.get(&page.to_string()).unwrap().clone();
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
                    id: "1".into(),
                    title: format!("Sebelumnnya (Halaman {})", page - 1),
                    total_episodes: None,
                })
            }

            if !is_latest {
                movie_list.push(Movie {
                    id: "2".into(),
                    title: format!("Selanjutnya (Halaman {})", page + 1),
                    total_episodes: None,
                })
            }

            let movie = input::choice(movie_list, false)?;
            if movie.id == "1" {
                page -= 1;
            } else if movie.id == "2" {
                page += 1;
            } else {
                return Ok(movie);
            }
        }
    }

    pub async fn extract_episode(&mut self, movie: Movie) -> anyhow::Result<Episode> {
        print!(
            "{} Memuat daftar episode '{}' .. ",
            "◆".blue(),
            movie.id.green()
        );
        stdout().flush()?;

        let id = movie.id.clone();

        if !self.episode_cache.contains_key(&id) {
            let item = self.extractor.get_episodes(movie).await?;
            self.episode_cache.insert(id.clone(), item);
        }

        let (episodes, meta) = self.episode_cache.get(&id).unwrap().to_owned();
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

        if let Some(ref thumb_url) = meta.thumb_url {
            show_image_thumb(thumb_url.to_string()).await;
        }

        let mut has_episode = false;
        for (k, v) in meta.data {
            if k.to_lowercase().contains("episode") {
                has_episode = true;
            }
            println!("  {} : {}", k.bright_white(), v);
        }
        if !has_episode {
            println!("  {} : {}", "Episodes".bright_white(), episodes.len());
        }
        println!();

        let selected = input::choice(episodes, true)?;
        Ok(selected)
    }

    pub async fn extract_stream_urls(&mut self, episode: Episode) -> anyhow::Result<Stream> {
        print!(
            "{} Memuat tautan unduhan '{}' .. ",
            "◆".blue(),
            episode.id.green()
        );
        stdout().flush()?;

        let id = episode.id.clone();
        if !self.stream_cache.contains_key(&id) {
            let streams = self.extractor.get_stream_urls(episode).await?;
            self.stream_cache.insert(id.clone(), streams);
        }

        let streams = self.stream_cache.get(&id).unwrap().to_owned();
        if streams.is_empty() {
            eprintln!(
                "gagal!\n{} {}",
                "■".red(),
                "Silahkan jalankan ulang program\n".yellow()
            );
            std::process::exit(0);
        }
        println!("berhasil");

        let selected = input::choice(streams, false)?;
        Ok(selected)
    }
}
