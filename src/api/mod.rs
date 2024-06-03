use actix_web::{
    HttpResponse,
    Responder,
};

use std::env;

const DEFAULT_PORT: u16 = 8080;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

pub async fn check_version() -> impl Responder {
    HttpResponse::Ok().body(env!("CARGO_PKG_VERSION"))
}

pub fn get_server_port() -> u16 {
    env::var("APP_PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.to_string())
        .parse()
        .unwrap_or(DEFAULT_PORT)
}
