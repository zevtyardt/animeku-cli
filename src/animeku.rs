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
    movie_cache: HashMap<usize, Vec<Movie>>,
    pub episode_cache: HashMap<usize, (Vec<Episode>, Meta)>,
    extractor: Box<dyn Ext>,
}

impl AnimekuCli {
    pub fn new(extractor: Box<dyn Ext>) -> Self {
        Self {
            movie_cache: HashMap::new(),
            episode_cache: HashMap::new(),
            extractor,
        }
    }

    pub async fn search(&mut self, search_title: &str) -> anyhow::Result<Movie> {
        print!("{} Proses pencarian .. ", "◆".blue());
        stdout().flush()?;

        let mut page = 1;
        let mut is_latest = false;

        loop {
            if let std::collections::hash_map::Entry::Vacant(_) = self.movie_cache.entry(page) {
                let (movie_list, total) = self.extractor.search(search_title.into(), page).await?;
                if page == 1 && !movie_list.is_empty() {
                    println!("ditemukan {} judul anime", total.to_string().green());
                }
                self.movie_cache.insert(page, movie_list);
            }

            let mut movie_list = self.movie_cache.get(&page).unwrap().clone();
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

    pub async fn extract_episode(&mut self, movie: Movie) -> anyhow::Result<Episode> {
        print!("{} Mengambil semua daftar episode .. ", "◆".blue());
        stdout().flush()?;

        let channel_id = movie.channel_id as usize;
        let (episodes, meta) = if let Some(item) = self.episode_cache.get(&channel_id) {
            item.clone()
        } else {
            let item = self.extractor.get_episodes(movie).await?;
            self.episode_cache.insert(channel_id, item.clone());
            item
        };
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

        for (k, v) in meta.data {
            println!("  {:<9}: {}", k.bright_white(), v);
        }

        println!("  {:<9}: {}", "Episode".bright_white(), episodes.len());
        println!();

        let selected = input::choice(episodes, true)?;
        Ok(selected)
    }

    pub async fn extract_stream_urls(&self, episode: Episode) -> anyhow::Result<Stream> {
        print!("{} Memuat tautan unduhan .. ", "◆".green());
        stdout().flush()?;

        let streams = self.extractor.get_stream_urls(episode).await?;
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
