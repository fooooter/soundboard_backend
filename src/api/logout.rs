use drain_macros::*;
use drain_common::sessions::Session;
use drain_common::RequestData::Get;
use crate::api::error;

#[drain_endpoint("api/logout")]
pub fn is_logged_in() {
    match REQUEST_DATA {
        Get(_) => {
            let session: Session = start_session!().await;
            session.destroy().await;

            *HTTP_STATUS_CODE = 204u16;
            return None;
        },
        _ => {
            set_header!("Content-Type", "application/json");
            return error("This endpoint only accepts GET requests.", HTTP_STATUS_CODE, 400);
        }
    }
}