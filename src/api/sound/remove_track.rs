use std::env;
use drain_common::RequestData::Get;
use drain_common::sessions::Session;
use drain_macros::{drain_endpoint, set_header, start_session};
use serde_json::json;
use sqlx::{query, query_as, MySqlConnection, Connection};
use tokio::fs::remove_file;
use crate::api::{UserSession, Filename};

#[drain_endpoint("api/sound/remove_track")]
pub fn remove_track() {
    let session: Session = start_session!().await;

    let Some(mut user_id) = session.get::<UserSession>(&String::from("userId")).await else {
        set_header!("Content-Type", "application/json");
        return Some(Vec::from(json!({
            "error": "Please log in to use this endpoint."
        }).to_string()));
    };

    match REQUEST_DATA {
        Get(Some(data)) => {
            let Ok(conn_string) = env::var("MYSQL_CONN") else {
                return Some(Vec::from(json!({
                    "error": "\"MYSQL_CONN\" environment variable not found."
                }).to_string()));
            };

            let mut conn = match MySqlConnection::connect(&*conn_string).await {
                Ok(c) => c,
                Err(e) => {
                    return Some(Vec::from(json!({
                        "error": e.to_string()
                    }).to_string()));
                }
            };

            let Some(track_id) = data.get("id") else {
                set_header!("Content-Type", "application/json");
                return Some(Vec::from(json!({
                    "error": "Request does not contain a track ID parameter."
                }).to_string()));
            };

            let Some(filename): Option<Filename> = (match query_as("SELECT filename FROM sounds WHERE id = ? AND user_id = ?")
                .bind(track_id)
                .bind(user_id.id)
                .fetch_optional(&mut conn)
                .await {
                Ok(t) => t,
                Err(e) => {
                    return Some(Vec::from(json!({
                        "error": e.to_string()
                    }).to_string()));
                }
            }) else {
                set_header!("Content-Type", "application/json");
                return Some(Vec::from(json!({
                    "error": "File you're trying to delete doesn't exist."
                }).to_string()));
            };

            let Ok(sound_dir) = env::var("SOUND_DIR") else {
                return Some(Vec::from(json!({
                    "error": "\"SOUND_DIR\" environment variable not found."
                }).to_string()))
            };

            if let Err(e) = remove_file(format!("{sound_dir}/{}/{}", user_id.id, filename.filename)).await {
                set_header!("Content-Type", "application/json");
                return Some(Vec::from(json!({
                    "error": e.to_string()
                }).to_string()));
            }

            if let Err(e) = query("DELETE FROM sounds WHERE id = ? AND user_id = ?")
                .bind(track_id)
                .bind(user_id.id)
                .execute(&mut conn)
                .await {
                set_header!("Content-Type", "application/json");
                return Some(Vec::from(json!({
                    "error": e.to_string()
                }).to_string()));
            }

            return None;
        },
        _ => {
            set_header!("Content-Type", "application/json");
            return Some(Vec::from(json!({
                "error": "This endpoint only accepts GET requests containing a track ID parameter."
            }).to_string()));
        }
    }
}
