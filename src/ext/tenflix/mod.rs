use crate::{
    ext::Ext,
    models::{Episode, Meta, Movie, Stream},
    regex,
};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use html_escape::decode_html_entities;
use reqwest::Client;

pub struct TenflixExt {
    client: Client,
}

impl TenflixExt {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Ext for TenflixExt {
    async fn search(&mut self, title: String, page: usize) -> anyhow::Result<(Vec<Movie>, u64)> {
        let url = format!("https://tenflix.org/page/{}/?s={}", page, title);

        let response = self
            .client
            .get(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Host", "tenflix.org")
            .header("Connection", "Keep-Alive")
            .header("User-Agent", "okhttp/3.12.13")
            .send()
            .await?;

        let bytes = response.bytes().await?;
        let body = String::from_utf8_lossy(&bytes);

        let mut movies = vec![];

        let item = regex!(
            r#"<a\s*href="https://tenflix.org/([^"]+)">\s*([^<]+)\s*</a>.*?*</div>.*?<div class="meta">.*?<span class="year">(\d+)</span>"#
        );
        for cap in item.captures_iter(&body) {
            let id = &cap[1];
            let title = &cap[2];
            let year = &cap[3];

            let tipe = if id.starts_with("movie") {
                "Movie"
            } else {
                "TV"
            };
            let item = Movie {
                id: id.to_string(),
                title: format!("{} {} ({})", decode_html_entities(&title), year, tipe),
                total_episodes: None,
            };
            movies.push(item);
        }

        let len = movies.len() as u64;
        Ok((movies, len))
    }

    async fn get_episodes(&self, movie: Movie) -> anyhow::Result<(Vec<Episode>, Meta)> {
        let url = format!("https://tenflix.org/{}", movie.id);
        let response = self
            .client
            .get(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Host", "tenflix.org")
            .header("Connection", "Keep-Alive")
            .header("User-Agent", "okhttp/3.12.13")
            .send()
            .await?;

        let bytes = response.bytes().await?;
        let body = String::from_utf8_lossy(&bytes);

        let mut episodes = vec![];
        if movie.id.starts_with("tv") {
            let re = regex!(
                r#"numerando['"]>\s*([^>]+)\s*<.*?episodiotitle.*?href=['"]https:\/\/tenflix.org\/([^>]+)['"]\s*>\s*([^>]+)\s*<"#
            );

            for cap in re.captures_iter(&body) {
                let id = &cap[2];
                let title = format!("Season {}: {}", &cap[1], &cap[3]);

                let item = Episode {
                    id: id.into(),
                    title,
                    is_series: true,
                };
                episodes.push(item);
            }
        } else {
            episodes.push(Episode {
                id: movie.id,
                title: movie.title,
                is_series: true,
            })
        }

        let mut meta = Meta::default();
        let thumb_re = regex!(r#"poster['"]\s*>.*?image.*?src=['"]([^'"]+)"#);
        if let Some(cap) = thumb_re.captures(&body) {
            let url = &cap[1];
            meta.thumb_url = Some(url.to_string());
        }

        let meta_re = regex!(
            r#"custom_fields['"]\s*>.*?variante.*?>\s*([^>]+)\s*<.*?valor['"]\s*>.*?([^>]+)\s*<\/span"#
        );
        for cap in meta_re.captures_iter(&body) {
            let k = &cap[1];
            let v = &cap[2];
            if !v.trim().is_empty() {
                meta.data.push((k.to_string(), v.to_string()));
            }
        }

        Ok((episodes, meta))
    }

    async fn get_stream_urls(&self, episode: Episode) -> anyhow::Result<Vec<Stream>> {
        let mut streams = vec![];
        let url = format!("https://tenflix.org/{}", episode.id);
        if let Some(embed_url) = get_download_link(&self.client, url).await? {
            let response = self
                .client
                .get(embed_url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Connection", "Keep-Alive")
                .header("User-Agent", "okhttp/3.12.13")
                .send()
                .await?;

            let bytes = response.bytes().await?;
            let body = String::from_utf8_lossy(&bytes);

            let re3 = regex!(r#"item.*?id=['"]([^'"]+).*?data-frame=['"]([^'"]+).*?>([^<]+)<"#);
            for cap in re3.captures_iter(&body) {
                if let Ok(mut url) = String::from_utf8(STANDARD.decode(&cap[2])?) {
                    if cap[1].contains("priv") {
                        url = extract_private_server(&self.client, url).await;
                    } else if !check_server(&self.client, url.clone()).await {
                        continue;
                    }

                    let item = Stream {
                        title: format!("{}: {}", &cap[1], &cap[3]),
                        url,
                    };

                    streams.push(item);
                }
            }
        }

        Ok(streams)
    }
}

async fn check_server(client: &Client, url: String) -> bool {
    if let Ok(response) = client
        .head(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Connection", "Keep-Alive")
        .header("User-Agent", "okhttp/3.12.13")
        .send()
        .await
    {
        return response.status().is_success();
    }

    true
}

async fn extract_private_server(client: &Client, url: String) -> String {
    if let Ok(response) = client
        .get(url.clone())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Connection", "Keep-Alive")
        .header("User-Agent", "okhttp/3.12.13")
        .send()
        .await
    {
        if let Ok(bytes) = response.bytes().await {
            let body = String::from_utf8_lossy(&bytes);
            let re = regex!(r#"file\s*:\s*['"]([^'"]+)"#);

            if let Some(cap) = re.captures(&body) {
                let m3url = &cap[1];
                return m3url.to_string();
            }
        }
    }

    url
}

async fn get_embed_link(client: &Client, url: &str) -> anyhow::Result<Option<String>> {
    let response = client
        .get(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Connection", "Keep-Alive")
        .header("User-Agent", "okhttp/3.12.13")
        .send()
        .await?;
    let bytes = response.bytes().await?;
    let body = String::from_utf8_lossy(&bytes);

    let re = regex!(r#"href=['"](https:\/\/kotakajaib.me\/file\/[^/'"]+)"#);
    if let Some(cap) = re.captures(&body) {
        let embed = &cap[1].replace("file", "embed");
        return Ok(Some(embed.to_string()));
    }
    Ok(None)
}

async fn get_download_link(client: &Client, url: String) -> anyhow::Result<Option<String>> {
    let response = client
        .get(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Connection", "Keep-Alive")
        .header("User-Agent", "okhttp/3.12.13")
        .send()
        .await?;
    let bytes = response.bytes().await?;
    let body = String::from_utf8_lossy(&bytes);

    let re = regex!(r#"href=['"](https:\/\/tenflix.org\/links/[^/'"]+)"#);
    for cap in re.captures_iter(&body) {
        let url = &cap[1];
        if let Some(embed) = get_embed_link(client, url).await? {
            return Ok(Some(embed));
        }
    }
    Ok(None)
}
