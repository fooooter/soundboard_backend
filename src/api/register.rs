use drain_common::RequestBody::XWWWFormUrlEncoded;
use drain_common::RequestData::*;
use drain_macros::*;
use openssl::base64;
use openssl::hash::{hash, MessageDigest};
use sqlx::{query, query_as};
use crate::api::{error, Username};
use crate::connection::get_connection;

#[drain_endpoint("api/register")]
pub fn register() {
    set_header!("Content-Type", "application/json");

    match REQUEST_DATA {
        Post { data: Some(XWWWFormUrlEncoded(data)), .. } => {
            let login = data.get("login");
            let password = data.get("password");

            match (login, password) {
                (Some(login), Some(password)) if !login.is_empty() && !password.is_empty() => {
                    let mut conn = match get_connection().await {
                        Ok(conn) => conn,
                        Err(e) => {
                            return error(&*e, HTTP_STATUS_CODE, 500);
                        }
                    };

                    let usernames: Vec<Username> = match query_as("SELECT username FROM users WHERE username = ?")
                        .bind(login)
                        .fetch_all(&mut *conn)
                        .await {
                        Ok(t) => t,
                        Err(e) => {
                            return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                        }
                    };

                    let username = Username {
                        username: login.clone()
                    };

                    if usernames.contains(&username) {
                        return error("The username you've provided already exists.", HTTP_STATUS_CODE, 200);
                    }

                    let password_hash = base64::encode_block(&*hash(MessageDigest::sha256(), password.as_bytes()).unwrap());

                    if let Err(e) = query("INSERT INTO users (username, password) VALUES (?, ?)")
                        .bind(login)
                        .bind(password_hash)
                        .execute(&mut *conn)
                        .await {
                        return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                    }

                    if let Err(e) = conn.close().await {
                        return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                    }

                    return None;
                },
                _ => {
                    return error("Login and password have to be present.", HTTP_STATUS_CODE, 400);
                }
            }
        },
        _ => {
            return error("This endpoint only accepts POST requests.", HTTP_STATUS_CODE, 400);
        }
    }
}