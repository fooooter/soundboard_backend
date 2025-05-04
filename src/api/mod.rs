use drain_macros::SessionValue;
use drain_common::sessions::SessionValue;
use serde_json::json;
use sqlx::FromRow;

mod login;
mod register;
mod is_logged_in;
mod logout;

mod sound;

#[derive(SessionValue, Clone)]
struct UserSession {
    pub id: u32
}

#[derive(FromRow)]
struct UserID {
    pub id: u32
}

#[derive(FromRow, PartialEq)]
struct Username {
    pub username: String
}

#[derive(FromRow)]
struct Filename {
    pub filename: String
}

fn error(e: &str, orig_status: &mut u16, status: u16) -> Option<Vec<u8>> {
    *orig_status = status;
    Some(Vec::from(json!({
        "error": e
    }).to_string()))
}