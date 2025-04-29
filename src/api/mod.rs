use drain_macros::SessionValue;
use drain_common::sessions::SessionValue;
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