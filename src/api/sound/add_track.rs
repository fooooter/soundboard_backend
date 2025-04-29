use crate::api::UserSession;
use drain_common::sessions::Session;
use drain_common::RequestData::Post;
use drain_common::RequestBody::FormData;
use drain_macros::{drain_endpoint, set_header, start_session};
use sqlx::{query, MySqlConnection, Connection};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use serde_json::json;

#[drain_endpoint("api/sound/add_track")]
pub fn add_track() {
    set_header!("Content-Type", "application/json");
    let session: Session = start_session!().await;

    let Some(mut user_id) = session.get::<UserSession>(&String::from("userId")).await else {
        return Some(Vec::from(json!({
            "error": "Please log in to use this endpoint."
        }).to_string()));
    };

    match REQUEST_DATA {
        Post { data: Some(FormData(data)), .. } => {
            let mut conn = match MySqlConnection::connect("mysql://root:@localhost:3306/music_player").await {
                Ok(c) => c,
                Err(e) => {
                    return Some(Vec::from(json!({
                        "error": e.to_string()
                    }).to_string()));
                }
            };

            let Some(track) = data.get("track") else {
                return Some(Vec::from(json!({
                    "error": "\"track\" field not present in request's body."
                }).to_string()));
            };

            if track.value[0] != 0x49 && track.value[1] != 0x44 && track.value[2] != 0x33 &&
               track.value[0] != 0xff && track.value[1] != 0xfb {
                return Some(Vec::from(json!({
                    "error": "The provided file does not contain valid MP3 data."
                }).to_string()));
            }

            let Some(mut filename) = track.filename.clone() else {
                return Some(Vec::from(json!({
                    "error": "Filename must be given."
                }).to_string()))
            };

            if let Some(f) = filename.rsplit_once('/') {
                filename = String::from(f.1);
            }

            let mut file = match File::create(&filename).await {
                Ok(f) => f,
                Err(e) => {
                    return Some(Vec::from(json!({
                        "error": e.to_string()
                    }).to_string()));
                }
            };

            if let Err(e) = file.write_all(&*track.value).await {
                return Some(Vec::from(json!({
                    "error": e.to_string()
                }).to_string()));
            }

            if let Err(e) = query("INSERT INTO sounds (filename, user_id) VALUES (?, ?)")
                .bind(filename)
                .bind(user_id.id)
                .execute(&mut conn)
                .await {
                return Some(Vec::from(json!({
                    "error": e.to_string()
                }).to_string()));
            }

            return None;
        },
        _ => {
            return Some(Vec::from(json!({
                "error": "This endpoint only accepts POST requests containing files."
            }).to_string()));
        }
    }
}
