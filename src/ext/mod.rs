use async_trait::async_trait;

use crate::models::{Episode, Meta, Movie, Stream};

pub mod nontonanime;

#[async_trait]
pub trait Ext {
    async fn search(&mut self, title: String, page: usize) -> anyhow::Result<(Vec<Movie>, u64)>;
    async fn get_episodes(&self, movie: Movie) -> anyhow::Result<(Vec<Episode>, Meta)>;
    async fn get_stream_urls(&self, episode: Episode) -> anyhow::Result<Vec<Stream>>;
}
