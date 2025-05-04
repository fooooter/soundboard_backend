use sqlx::{query_as, Error, MySqlConnection, Connection};
use std::env;
use openssl::hash::{MessageDigest, hash};
use drain_common::RequestBody::XWWWFormUrlEncoded;
use drain_common::RequestData::*;
use drain_common::sessions::Session;
use drain_macros::*;
use openssl::base64;
use crate::api::{UserSession, UserID, error};

#[drain_endpoint("api/login")]
pub fn login() {
    set_header!("Content-Type", "application/json");

    match REQUEST_DATA {
        Post { data: Some(XWWWFormUrlEncoded(data)), .. } => {
            let login = data.get("login");
            let password = data.get("password");

            match (login, password) {
                (Some(login), Some(password)) if !login.is_empty() && !password.is_empty() => {
                    let Ok(conn_string) = env::var("MYSQL_CONN") else {
                        return error("\"MYSQL_CONN\" environment variable not found.", HTTP_STATUS_CODE, 500);
                    };

                    let mut conn = match MySqlConnection::connect(&*conn_string).await {
                        Ok(c) => c,
                        Err(e) => {
                            return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                        }
                    };

                    let password_hash = base64::encode_block(&*hash(MessageDigest::sha256(), password.as_bytes()).unwrap());

                    let user: Result<Option<UserID>, Error> = query_as("SELECT id FROM users WHERE username = ? AND password = ?")
                        .bind(login)
                        .bind(password_hash)
                        .fetch_optional(&mut conn)
                        .await;

                    match user {
                        Ok(user) => {
                            let Some(UserID {id}) = user else {
                                return error("Invalid credentials.", HTTP_STATUS_CODE, 200);
                            };

                            let mut session: Session = start_session!().await;
                            session.set(String::from("userId"), Box::new(UserSession { id })).await;

                            return None;
                        },
                        Err(e) => {
                            return error(&*e.to_string(), HTTP_STATUS_CODE, 500);
                        }
                    }
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