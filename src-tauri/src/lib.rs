mod commands;
mod db;
mod error;
mod media;
mod models;
mod nodes;
mod sanitize;
mod search;
mod state;
mod topics;

use std::fs;

use tauri::http::{Response, StatusCode, header};
use tauri::{Manager, UriSchemeContext};

use crate::state::AppState;

/// Serves stored topic files (`media://localhost/<id>`) so that images embedded in a topic
/// can be shown by the webview without exposing the file system.
fn serve_media<R: tauri::Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let state = ctx.app_handle().state::<AppState>();
    let id = request.uri().path().trim_start_matches('/');

    let file = media::find(&state.conn(), &state.files_dir, id)
        .ok()
        .and_then(|file| fs::read(&file.path).ok().map(|bytes| (file, bytes)));

    match file {
        Some((file, bytes)) => {
            let disposition = if file.collection == media::IMAGES {
                "inline"
            } else {
                "attachment"
            };
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, file.mime_type)
                .header(header::CONTENT_DISPOSITION, disposition)
                .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
                .header(
                    header::CONTENT_SECURITY_POLICY,
                    "default-src 'none'; style-src 'unsafe-inline'; sandbox",
                )
                .body(bytes)
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Vec::new()),
    }
    .expect("static response parts are valid")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .register_uri_scheme_protocol("media", serve_media)
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let files_dir = data_dir.join("files");
            fs::create_dir_all(&files_dir)?;

            let conn = db::open(&data_dir.join("notes.db"))?;
            app.manage(AppState::new(conn, files_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_nodes,
            commands::create_node,
            commands::rename_node,
            commands::move_node,
            commands::delete_node,
            commands::list_topics,
            commands::get_topic,
            commands::create_topic,
            commands::update_topic,
            commands::delete_topic,
            commands::paste_topic,
            commands::toggle_favorite,
            commands::known_tags,
            commands::search_topics,
            commands::add_attachment,
            commands::add_image,
            commands::delete_media,
            commands::export_media,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Structured Notes");
}
