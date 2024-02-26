use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use crate::models::{Episode, Meta, Movie};

use super::Ext;

pub struct AnimeExt {
    pub client: Client,
}

impl AnimeExt {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            client: Client::new(),
        })
    }
}

#[async_trait]
impl Ext for AnimeExt {
    async fn search(&mut self, title: String, page: usize) -> anyhow::Result<(Vec<Movie>, u64)> {
        let url =
            "https://animeku.my.id/nontonanime-v77/phalcon/api/search_category_collection/v7_1/";
        let payload = format!(
            "search={}&page={}&count=20&lang=All&isAPKvalid=true",
            title, page
        );

        let response = self
            .client
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

        let mut items = Vec::new();
        let json = response.json::<Value>().await?;

        if let Some(categories) = json["categories"].as_array() {
            for cat in categories {
                if let Some(channel_id) = cat["cid"].as_u64() {
                    if [1, 2].contains(&channel_id) {
                        continue;
                    }
                    let item = Movie {
                        channel_id,
                        title: cat["category_name"].to_string().trim_matches('"').into(),
                        total_episodes: Some(
                            cat["count_anime"].as_str().unwrap_or("1").to_string(),
                        ),
                    };

                    items.push(item)
                }
            }
        }
        let total = json["count_total"].as_u64().unwrap_or(0);
        Ok((items, total))
    }

    async fn get_episodes(&self, movie: Movie) -> anyhow::Result<(Vec<Episode>, Meta)> {
        let url =
            "https://animeku.my.id/nontonanime-v77/phalcon/api/get_category_posts_secure/v9_1/";
        let payload = format!("id={}&isAPKvalid=true", movie.channel_id);

        let response = self
            .client
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

        let mut episodes = Vec::new();
        let mut meta = Meta::default();

        let json = response.json::<Value>().await?;

        if let Some(category) = json["category"].as_object() {
            if let Some(thumb) = category.get("img_url") {
                meta.thumb_url = Some(thumb.to_string().trim_matches('"').trim().into());
            }
            for (key, value) in [
                ("Judul", "category_name"),
                ("Genre", "genre"),
                ("Tahun", "years"),
                ("Rating", "rating"),
            ] {
                if let Some(v) = category.get(value) {
                    meta.data
                        .push((key.into(), v.to_string().trim_matches('"').trim().into()));
                }
            }
            if let Some(v) = category["ongoing"].as_u64() {
                meta.data.push(("On-Going".into(), (v != 0).to_string()));
            }
        }

        if let Some(posts) = json["posts"].as_array() {
            for post in posts {
                if let Some(channel_id) = post["channel_id"].as_u64() {
                    let item = Episode {
                        channel_id,
                        category_id: post["category_id,"].as_u64().unwrap(),
                        title: post["channel_name"]
                            .to_string()
                            .trim_matches('"')
                            .trim()
                            .into(),
                        is_anime: true,
                    };
                    episodes.push(item);
                }
            }
        }

        Ok((episodes, meta))
    }
}
