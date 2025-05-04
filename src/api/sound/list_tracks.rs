use std::env;
use drain_common::RequestData::Get;
use drain_common::sessions::Session;
use drain_macros::{drain_endpoint, set_header, start_session};
use serde::Serialize;
use sqlx::{FromRow, query_as, Connection, MySqlConnection};
use crate::api::{error, UserSession};

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
        return error("Please log in to use this endpoint.", HTTP_STATUS_CODE, 401);
    };

    match REQUEST_DATA {
        Get(_) => {
            let Ok(conn_string) = env::var("MYSQL_CONN") else {
                return error("\"MYSQL_CONN\" environment variable not found.", HTTP_STATUS_CODE, 500);
            };

            let mut conn = match MySqlConnection::connect(&*conn_string).await {
                Ok(c) => c,
                Err(e) => {
                    return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                }
            };

            let tracks: Vec<Track> = match query_as("SELECT id, filename FROM sounds WHERE user_id = ? ORDER BY id ASC")
                .bind(user_id.id)
                .fetch_all(&mut conn)
                .await {
                Ok(t) => t,
                Err(e) => {
                    return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                }
            };

            match serde_json::to_vec(&tracks) {
                Ok(json) => {
                    return Some(json);
                },
                Err(e) => {
                    return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                }
            }
        },
        _ => {
            return error("This endpoint only accepts GET requests.", HTTP_STATUS_CODE, 400);
        }
    }
}