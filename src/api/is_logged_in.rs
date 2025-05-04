use drain_macros::*;
use drain_common::sessions::Session;
use drain_common::RequestData::Get;
use serde_json::json;
use crate::api::{error, UserSession};

#[drain_endpoint("api/is_logged_in")]
pub fn is_logged_in() {
    set_header!("Content-Type", "application/json");
    match REQUEST_DATA {
        Get(_) => {
            let session: Session = start_session!().await;
            if session.get::<UserSession>(&String::from("userId")).await.is_none() {
                return Some(Vec::from(json!({
                    "loggedIn": false
                }).to_string()));
            }
            return Some(Vec::from(json!({
                "loggedIn": true
            }).to_string()))
        },
        _ => {
            return error("This endpoint only accepts GET requests.", HTTP_STATUS_CODE, 400);
        }
    }
}