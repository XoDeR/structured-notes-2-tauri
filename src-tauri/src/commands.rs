use std::path::PathBuf;

use tauri::State;
use tauri::ipc::{InvokeBody, Request};

use crate::error::{Error, Result};
use crate::models::{
    Attachment, PasteMode, SearchQuery, SearchResult, TopicDetail, TopicInput, TopicSummary,
    TreeNode,
};
use crate::state::AppState;
use crate::{media, nodes, search, topics};

#[tauri::command]
pub async fn list_nodes(state: State<'_, AppState>) -> Result<Vec<TreeNode>> {
    nodes::list(&state.conn())
}

#[tauri::command]
pub async fn create_node(
    state: State<'_, AppState>,
    parent_id: String,
    title: String,
) -> Result<String> {
    nodes::create(&state.conn(), &parent_id, &title)
}

#[tauri::command]
pub async fn rename_node(state: State<'_, AppState>, id: String, title: String) -> Result<()> {
    nodes::rename(&state.conn(), &id, &title)
}

#[tauri::command]
pub async fn move_node(state: State<'_, AppState>, id: String, parent_id: String) -> Result<()> {
    nodes::move_to(&state.conn(), &id, &parent_id)
}

/// Returns the parent of the deleted node, which becomes the selection.
#[tauri::command]
pub async fn delete_node(state: State<'_, AppState>, id: String) -> Result<String> {
    nodes::delete(&state.conn(), &state.files_dir, &id)
}

#[tauri::command]
pub async fn list_topics(state: State<'_, AppState>, node_id: String) -> Result<Vec<TopicSummary>> {
    topics::list_in_node(&state.conn(), &node_id)
}

#[tauri::command]
pub async fn get_topic(
    state: State<'_, AppState>,
    id: String,
    node_id: Option<String>,
) -> Result<Option<TopicDetail>> {
    topics::detail(&state.conn(), &id, node_id.as_deref())
}

#[tauri::command]
pub async fn create_topic(
    state: State<'_, AppState>,
    node_id: String,
    input: TopicInput,
) -> Result<String> {
    topics::create(&state.conn(), &node_id, &input)
}

#[tauri::command]
pub async fn update_topic(state: State<'_, AppState>, id: String, input: TopicInput) -> Result<()> {
    topics::update(&state.conn(), &id, &input)
}

#[tauri::command]
pub async fn delete_topic(state: State<'_, AppState>, id: String, node_id: String) -> Result<()> {
    topics::delete(&state.conn(), &state.files_dir, &id, &node_id)
}

#[tauri::command]
pub async fn paste_topic(
    state: State<'_, AppState>,
    id: String,
    mode: PasteMode,
    node_id: String,
    from_node_id: Option<String>,
) -> Result<String> {
    topics::paste(
        &state.conn(),
        &state.files_dir,
        &id,
        mode,
        &node_id,
        from_node_id.as_deref(),
    )
}

#[tauri::command]
pub async fn toggle_favorite(state: State<'_, AppState>, id: String) -> Result<()> {
    topics::toggle_favorite(&state.conn(), &id)
}

#[tauri::command]
pub async fn known_tags(state: State<'_, AppState>) -> Result<Vec<String>> {
    topics::known_tags(&state.conn())
}

#[tauri::command]
pub async fn search_topics(
    state: State<'_, AppState>,
    query: SearchQuery,
) -> Result<Vec<SearchResult>> {
    search::run(&state.conn(), &query)
}

#[tauri::command]
pub async fn add_attachment(
    state: State<'_, AppState>,
    topic_id: String,
    path: PathBuf,
) -> Result<Attachment> {
    media::add_attachment(&state.conn(), &state.files_dir, &topic_id, &path)
}

/// Receives the raw image bytes as the request body; the topic and file name travel in headers.
#[tauri::command]
pub async fn add_image(state: State<'_, AppState>, request: Request<'_>) -> Result<String> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err(Error::invalid("Expected raw image data."));
    };
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
            .ok_or_else(|| Error::invalid(format!("Missing {name} header.")))
    };
    let topic_id = header("x-topic-id")?;
    let file_name = percent_encoding::percent_decode_str(&header("x-file-name")?)
        .decode_utf8_lossy()
        .into_owned();

    media::add_image(
        &state.conn(),
        &state.files_dir,
        &topic_id,
        &file_name,
        bytes,
    )
}

#[tauri::command]
pub async fn delete_media(state: State<'_, AppState>, id: String) -> Result<()> {
    media::delete(&state.conn(), &state.files_dir, &id)
}

#[tauri::command]
pub async fn export_media(state: State<'_, AppState>, id: String, path: PathBuf) -> Result<()> {
    media::export(&state.conn(), &state.files_dir, &id, &path)
}
