use async_trait::async_trait;

use crate::models::{Episode, Meta, Movie};

pub mod anime;
pub mod movie;

#[async_trait]
pub trait Ext {
    async fn search(&mut self, title: String, page: usize) -> anyhow::Result<(Vec<Movie>, u64)>;
    async fn get_episodes(&self, movie: Movie) -> anyhow::Result<(Vec<Episode>, Meta)>;
}
