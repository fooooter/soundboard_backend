use serde_json::json;
use drain_common::RequestBody::XWWWFormUrlEncoded;
use drain_common::RequestData::*;
use drain_macros::*;
use openssl::base64;
use openssl::hash::{hash, MessageDigest};
use sqlx::{query, query_as, Connection, MySqlConnection};
use crate::api::Username;

#[drain_endpoint("api/register")]
pub fn register() {
    set_header!("Content-Type", "application/json");

    match REQUEST_DATA {
        Post { data: Some(XWWWFormUrlEncoded(data)), .. } => {
            let login = data.get("login");
            let password = data.get("password");

            match (login, password) {
                (Some(login), Some(password)) if !login.is_empty() && !password.is_empty() => {
                    let mut conn = MySqlConnection::connect("mysql://root:@localhost:3306/soundboard" /* example connection string */).await.unwrap();

                    let usernames: Vec<Username> = match query_as("SELECT username FROM users WHERE username = ?")
                        .bind(login)
                        .fetch_all(&mut conn)
                        .await {
                        Ok(t) => t,
                        Err(e) => {
                            return Some(Vec::from(json!({
                                "error": e.to_string()
                            }).to_string()));
                        }
                    };

                    let username = Username {
                        username: login.clone()
                    };

                    if usernames.contains(&username) {
                        return Some(Vec::from(json!({
                            "error": "The username you've provided already exists."
                        }).to_string()));
                    }

                    let password_hash = base64::encode_block(&*hash(MessageDigest::sha256(), password.as_bytes()).unwrap());

                    if let Err(e) = query("INSERT INTO users (username, password) VALUES (?, ?)")
                        .bind(login)
                        .bind(password_hash)
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