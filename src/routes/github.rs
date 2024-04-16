use axum::{extract::{Request, State}, http::StatusCode};

use crate::config::Config;

pub async fn handle_gh(state: State<Config>, req: Request) -> StatusCode {
    let headers = req.headers();

    println!("{:#?}", headers);

    StatusCode::OK
}
