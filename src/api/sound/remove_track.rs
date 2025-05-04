use std::env;
use drain_common::RequestData::Get;
use drain_common::sessions::Session;
use drain_macros::{drain_endpoint, set_header, start_session};
use sqlx::{query, query_as, MySqlConnection, Connection};
use tokio::fs::remove_file;
use crate::api::{UserSession, Filename, error};

#[drain_endpoint("api/sound/remove_track")]
pub fn remove_track() {
    let session: Session = start_session!().await;

    let Some(mut user_id) = session.get::<UserSession>(&String::from("userId")).await else {
        set_header!("Content-Type", "application/json");
        return error("Please log in to use this endpoint.", HTTP_STATUS_CODE, 401);
    };

    match REQUEST_DATA {
        Get(Some(data)) => {
            let Ok(conn_string) = env::var("MYSQL_CONN") else {
                set_header!("Content-Type", "application/json");
                return error("\"MYSQL_CONN\" environment variable not found.", HTTP_STATUS_CODE, 500);
            };

            let mut conn = match MySqlConnection::connect(&*conn_string).await {
                Ok(c) => c,
                Err(e) => {
                    set_header!("Content-Type", "application/json");
                    return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                }
            };

            let Some(track_id) = data.get("id") else {
                set_header!("Content-Type", "application/json");
                return error("Request does not contain a track ID parameter.", HTTP_STATUS_CODE, 400);
            };

            let Some(filename): Option<Filename> = (match query_as("SELECT filename FROM sounds WHERE id = ? AND user_id = ?")
                .bind(track_id)
                .bind(user_id.id)
                .fetch_optional(&mut conn)
                .await {
                Ok(t) => t,
                Err(e) => {
                    set_header!("Content-Type", "application/json");
                    return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                }
            }) else {
                set_header!("Content-Type", "application/json");
                return error("File you're trying to delete doesn't exist.", HTTP_STATUS_CODE, 404);
            };

            let Ok(sound_dir) = env::var("SOUND_DIR") else {
                set_header!("Content-Type", "application/json");
                return error("\"SOUND_DIR\" environment variable not found.", HTTP_STATUS_CODE, 500);
            };

            if let Err(e) = remove_file(format!("{sound_dir}/{}/{}", user_id.id, filename.filename)).await {
                set_header!("Content-Type", "application/json");
                return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
            }

            if let Err(e) = query("DELETE FROM sounds WHERE id = ? AND user_id = ?")
                .bind(track_id)
                .bind(user_id.id)
                .execute(&mut conn)
                .await {
                set_header!("Content-Type", "application/json");
                return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
            }

            return None;
        },
        _ => {
            set_header!("Content-Type", "application/json");
            return error("This endpoint only accepts GET requests containing a track ID parameter.", HTTP_STATUS_CODE, 400);
        }
    }
}
