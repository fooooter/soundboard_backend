use serde_json::json;
use sqlx::{query_as, Error, MySqlConnection, Connection};
use openssl::hash::{MessageDigest, hash};
use drain_common::RequestBody::XWWWFormUrlEncoded;
use drain_common::RequestData::*;
use drain_common::sessions::Session;
use drain_macros::*;
use openssl::base64;
use crate::api::{UserSession, UserID};

#[drain_endpoint("api/login")]
pub fn login() {
    set_header!("Content-Type", "application/json");

    match REQUEST_DATA {
        Post { data: Some(XWWWFormUrlEncoded(data)), .. } => {
            let login = data.get("login");
            let password = data.get("password");

            match (login, password) {
                (Some(login), Some(password)) if !login.is_empty() && !password.is_empty() => {
                    let mut conn = MySqlConnection::connect("mysql://root:@localhost:3306/soundboard" /* example connection string */).await.unwrap();

                    let password_hash = base64::encode_block(&*hash(MessageDigest::sha256(), password.as_bytes()).unwrap());

                    let user: Result<Option<UserID>, Error> = query_as("SELECT id FROM users WHERE username = ? AND password = ?")
                        .bind(login)
                        .bind(password_hash)
                        .fetch_optional(&mut conn)
                        .await;

                    match user {
                        Ok(user) => {
                            let Some(UserID {id}) = user else {
                                return Some(Vec::from(json!({
                                    "error": "Invalid credentials."
                                }).to_string()));
                            };

                            let mut session: Session = start_session!().await;
                            session.set(String::from("userId"), Box::new(UserSession { id })).await;

                            return None;
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
                        "error": "Login and password have to be present."
                    }).to_string()));
                }
            }
        },
        _ => {
            return Some(Vec::from(json!({
                "error": "This endpoint only accepts POST requests."
            }).to_string()));
        }
    }
}