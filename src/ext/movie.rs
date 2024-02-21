use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use crate::models::{Episode, Meta, Movie};

use super::Ext;

pub struct MovieExt {
    pub client: Client,
    metadata: HashMap<u64, Meta>,
}

impl MovieExt {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            client: Client::new(),
            metadata: HashMap::new(),
        })
    }
}

#[async_trait]
impl Ext for MovieExt {
    async fn search(&mut self, title: String, page: usize) -> anyhow::Result<(Vec<Movie>, u64)> {
        let url = "https://animeku.my.id/nontonanime-v77/phalcon/api/search_anime_movie/v7_1/";
        let payload = format!(
            "search={}&page={}&count=20&lang=All&isAPKvalid=true",
            title, page
        );

        let response = self
            .client
            .post(url)
            .header("Cache-Control", "max-age=0")
            .header("Data-Agent", "New Aniplex v9.1")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Content-Length", payload.len())
            .header("Host", "animeku.my.id")
            .header("Connection", "Keep-Alive")
            .header("Accept-Encoding", "gzip")
            .header("User-Agent", "okhttp/3.12.13")
            .body(payload)
            .send()
            .await?;

        let mut items = Vec::new();
        let json = response.json::<Value>().await?;

        /*

                 Object {
            "category_name": String(" Anime Movie Sub Indo"),
            "channel_id": Number(25107),
            "channel_image": String("1578492231_kimi-to-nami-ni-noretara.jpg"),
            "channel_name": String("Kimi to Nami ni Noretara Movie"),
            "count_view": String("65856"),
            "img_url": String("https://nontonanime.b-cdn.net/nontonanime/upload/1578492231_kimi-to-nami-ni-noretara.jpg"),
            "lang": String("ID"),
            "ongoing": Number(0),
            "rating": String("9.00"),
            "safe_images": Bool(false),
            "years": Number(2021),
        }

                 */
        if let Some(posts) = json["posts"].as_array() {
            for post in posts {
                if let Some(channel_id) = post["channel_id"].as_u64() {
                    let mut meta = Meta {
                        thumb_url: post
                            .get("img_url")
                            .map(|v| v.to_string().trim_matches('"').trim().into()),
                        data: Vec::new(),
                    };

                    for (key, value) in [
                        ("Judul", "channel_name"),
                        ("Language", "lamg"),
                        ("Rating", "rating"),
                        ("years", "years"),
                    ] {
                        if let Some(v) = post.get(value) {
                            meta.data
                                .push((key.into(), v.to_string().trim_matches('"').trim().into()))
                        }
                    }
                    self.metadata.insert(channel_id, meta);

                    let item = Movie {
                        channel_id,
                        title: post["channel_name"]
                            .to_string()
                            .trim_matches('"')
                            .trim()
                            .into(),
                        total_episodes: None,
                    };
                    items.push(item)
                }
            }
        }
        let total = items.len() as u64;
        Ok((items, total))
    }

    async fn get_episodes(&self, movie: Movie) -> anyhow::Result<(Vec<Episode>, Meta)> {
        let item = Episode {
            category_id: 0,
            channel_id: movie.channel_id,
            title: movie.title,
        };

        if let Some(v) = self.metadata.get(&movie.channel_id) {
            Ok((vec![item], v.clone()))
        } else {
            Ok((vec![item], Meta::default()))
        }
    }
}
