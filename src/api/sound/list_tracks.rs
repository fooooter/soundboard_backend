use drain_common::RequestData::Get;
use drain_common::sessions::Session;
use drain_macros::{drain_endpoint, set_header, start_session};
use serde::Serialize;
use serde_json::json;
use sqlx::{FromRow, query_as, Connection, MySqlConnection};
use crate::api::UserSession;

#[derive(FromRow, Serialize)]
struct Track {
    id: u32,
    filename: String
}

#[drain_endpoint("api/sound/list_tracks")]
pub fn list_tracks() {
    set_header!("Content-Type", "application/json");
    let session: Session = start_session!().await;

    let Some(mut user_id) = session.get::<UserSession>(&String::from("userId")).await else {
        return Some(Vec::from(json!({
            "error": "Please log in to use this endpoint."
        }).to_string()));
    };

    match REQUEST_DATA {
        Get(_) => {
            let mut conn = match MySqlConnection::connect("mysql://root:@localhost:3306/soundboard" /* example connection string */).await {
                Ok(c) => c,
                Err(e) => {
                    return Some(Vec::from(json!({
                        "error": e.to_string()
                    }).to_string()));
                }
            };

            let tracks: Vec<Track> = match query_as("SELECT id, filename FROM sounds WHERE user_id = ? ORDER BY id ASC")
                .bind(user_id.id)
                .fetch_all(&mut conn)
                .await {
                Ok(t) => t,
                Err(e) => {
                    return Some(Vec::from(json!({
                        "error": e.to_string()
                    }).to_string()));
                }
            };

            match serde_json::to_vec(&tracks) {
                Ok(json) => {
                    return Some(json);
                },
                Err(e) => {
                    return Some(Vec::from(json!({
                        "error": e.to_string()
                    }).to_string()));
                }
            }
        },
        _ => {
            return Some(Vec::from(json!({
                "error": "This endpoint only accepts GET requests."
            }).to_string()));
        }
    }
}