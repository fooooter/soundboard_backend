use std::env;
use crate::api::{UserSession, Filename, error};
use drain_common::sessions::Session;
use drain_common::RequestData::Get;
use drain_macros::{drain_endpoint, set_header, start_session};
use sqlx::query_as;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::connection::get_connection;

#[drain_endpoint("api/sound/get_track")]
pub fn get_track() {
    set_header!("Content-Type", "application/json");
    let session: Session = start_session!().await;

    let Some(mut user_id) = session.get::<UserSession>(&String::from("userId")).await else {
        return error("Please log in to use this endpoint.", HTTP_STATUS_CODE, 401);
    };

    match REQUEST_DATA {
        Get(Some(data)) => {
            let mut conn = match get_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    return error(&*e, HTTP_STATUS_CODE, 500);
                }
            };

            let Some(track_id) = data.get("id") else {
                return error("Request does not contain a track ID parameter.", HTTP_STATUS_CODE, 400);
            };

            let Some(filename): Option<Filename> = (match query_as("SELECT filename FROM sounds WHERE id = ? AND user_id = ?")
                .bind(track_id)
                .bind(user_id.id)
                .fetch_optional(&mut *conn)
                .await {
                Ok(t) => t,
                Err(e) => {
                    return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                }
            }) else {
                return error("Content not found.", HTTP_STATUS_CODE, 404);
            };

            if let Err(e) = conn.close().await {
                return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
            }

            let Ok(sound_dir) = env::var("SOUND_DIR") else {
                return error("\"SOUND_DIR\" environment variable not found.", HTTP_STATUS_CODE, 500);
            };

            let Ok(mut file) = File::open(format!("{sound_dir}/{}/{}", user_id.id, filename.filename)).await else {
                return error("Content not found.", HTTP_STATUS_CODE, 404);
            };

            let mut content = Vec::new();
            if let Err(e) = file.read_to_end(&mut content).await {
                return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
            }

            set_header!("Content-Type", "audio/mpeg");
            return Some(content);
        },
        _ => {
            return error("This endpoint only accepts GET requests containing a track ID parameter.", HTTP_STATUS_CODE, 400);
        }
    }
}