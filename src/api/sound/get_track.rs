use crate::api::{UserSession, Filename};
use drain_common::sessions::Session;
use drain_common::RequestData::Get;
use drain_macros::{drain_endpoint, set_header, start_session};
use sqlx::{query_as, MySqlConnection, Connection};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use serde_json::json;

#[drain_endpoint("api/sound/get_track")]
pub fn get_track() {
    set_header!("Content-Type", "application/json");
    let session: Session = start_session!().await;

    let Some(mut user_id) = session.get::<UserSession>(&String::from("userId")).await else {
        return Some(Vec::from(json!({
            "error": "Please log in to use this endpoint."
        }).to_string()));
    };

    match REQUEST_DATA {
        Get(Some(data)) => {
            let mut conn = match MySqlConnection::connect("mysql://root:@localhost:3306/soundboard" /* example connection string */).await {
                Ok(c) => c,
                Err(e) => {
                    return Some(Vec::from(json!({
                        "error": e.to_string()
                    }).to_string()));
                }
            };

            let Some(track_id) = data.get("id") else {
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
                return Some(Vec::from(json!({
                    "error": "Content not found."
                }).to_string()));
            };

            let Ok(mut file) = File::open(filename.filename).await else {
                return Some(Vec::from(json!({
                    "error": "Content not found."
                }).to_string()));
            };

            let mut content = Vec::new();
            if let Err(e) = file.read_to_end(&mut content).await {
                return Some(Vec::from(json!({
                    "error": e.to_string()
                }).to_string()));
            }

            set_header!("Content-Type", "audio/mpeg");
            return Some(content);
        },
        _ => {
            return Some(Vec::from(json!({
                "error": "This endpoint only accepts GET requests containing a track ID parameter."
            }).to_string()));
        }
    }
}