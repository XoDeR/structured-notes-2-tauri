use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct TreeNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    /// The root node is structural and cannot be edited, moved or removed.
    pub can_edit: bool,
}

#[derive(Debug, Serialize)]
pub struct TopicSummary {
    pub id: String,
    pub title: String,
    pub is_link: bool,
    pub is_favorite: bool,
}

#[derive(Debug, Serialize)]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub size: i64,
}

#[derive(Debug, Serialize)]
pub struct TopicDetail {
    pub id: String,
    pub title: String,
    pub content: String,
    pub is_link: bool,
    pub can_edit: bool,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Deserialize)]
pub struct TopicInput {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PasteMode {
    Move,
    Copy,
    Link,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    #[default]
    Phrase,
    All,
}

#[derive(Debug, Default, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub fav: bool,
    #[serde(default)]
    pub mode: SearchMode,
    #[serde(default)]
    pub title_only: bool,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub topic_id: String,
    pub node_id: String,
    pub title: String,
    pub path: String,
}
