use std::{env, fs};
use crate::api::{error, UserSession};
use drain_common::sessions::Session;
use drain_common::RequestData::Post;
use drain_common::RequestBody::FormData;
use drain_macros::{drain_endpoint, set_header, start_session};
use sqlx::{query, MySqlConnection, Connection};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[drain_endpoint("api/sound/add_track")]
pub fn add_track() {
    set_header!("Content-Type", "application/json");
    let session: Session = start_session!().await;

    let Some(mut user_id) = session.get::<UserSession>(&String::from("userId")).await else {
        return error("Please log in to use this endpoint.", HTTP_STATUS_CODE, 401);
    };

    match REQUEST_DATA {
        Post { data: Some(FormData(data)), .. } => {
            let Ok(conn_string) = env::var("MYSQL_CONN") else {
                return error("\"MYSQL_CONN\" environment variable not found.", HTTP_STATUS_CODE, 500);
            };

            let mut conn = match MySqlConnection::connect(&*conn_string).await {
                Ok(c) => c,
                Err(e) => {
                    return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                }
            };

            let Some(track) = data.get("track") else {
                return error("\"track\" field not present in request's body.", HTTP_STATUS_CODE, 400);
            };

            if track.value[0] != 0x49 && track.value[1] != 0x44 && track.value[2] != 0x33 &&
               track.value[0] != 0xff && track.value[1] != 0xfb {
                return error("The provided file does not contain valid MP3 data.", HTTP_STATUS_CODE, 400);
            }

            let Some(mut filename) = track.filename.clone() else {
                return error("Filename must be given.", HTTP_STATUS_CODE, 400);
            };

            if let Some(f) = filename.rsplit_once('/') {
                filename = String::from(f.1);
            }

            let Ok(sound_dir) = env::var("SOUND_DIR") else {
                return error("\"SOUND_DIR\" environment variable not found.", HTTP_STATUS_CODE, 500);
            };

            match fs::exists(format!("{sound_dir}/{}", user_id.id)) {
                Ok(exists) => {
                    if !exists {
                        if let Err(e) = fs::create_dir(format!("{sound_dir}/{}", user_id.id)) {
                            return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                        }
                    }
                },
                Err(e) => {
                    return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                }
            }

            let mut file = match File::create(format!("{sound_dir}/{}/{}", user_id.id, &filename)).await {
                Ok(f) => f,
                Err(e) => {
                    return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                }
            };

            if let Err(e) = file.write_all(&*track.value).await {
                return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
            }

            if let Err(e) = query("INSERT INTO sounds (filename, user_id) VALUES (?, ?)")
                .bind(filename)
                .bind(user_id.id)
                .execute(&mut conn)
                .await {
                return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
            }

            return None;
        },
        _ => {
            return error("This endpoint only accepts POST requests containing files.", HTTP_STATUS_CODE, 400);
        }
    }
}
