//! Image list service — server-side seed data (REST-style loader demo).

use resuma::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageItem {
    pub id: u32,
    pub title: String,
    pub thumb_url: String,
    pub full_url: String,
}

pub fn search(query: &str, limit: usize) -> Vec<ImageItem> {
    let q = query.trim().to_lowercase();
    let total = 120usize;
    (1..=total)
        .filter(|i| {
            q.is_empty()
                || format!("image {i}").contains(&q)
                || q.parse::<u32>().ok() == Some(*i as u32)
        })
        .take(limit)
        .map(|i| ImageItem {
            id: i as u32,
            title: format!("Image {i}"),
            thumb_url: format!("https://picsum.photos/seed/resuma{i}/160/120"),
            full_url: format!("https://picsum.photos/seed/resuma{i}/480/360"),
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageListData {
    pub query: String,
    pub items: Vec<ImageItem>,
}

#[load]
pub async fn audit_image_list(req: &FlowRequest) -> ImageListData {
    let query = req.query_param("q").unwrap_or("").trim().to_string();
    ImageListData {
        query: query.clone(),
        items: search(&query, 120),
    }
}
