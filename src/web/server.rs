use crate::executor::SimpleExecutor;
use crate::types::ExecutionPayload;
use actix_cors::Cors;
use actix_web::http;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::env;
use std::net::Ipv4Addr;

const DEFAULT_PORT: u16 = 8080;

async fn execute_code(payload: web::Json<ExecutionPayload>) -> impl actix_web::Responder {
    let result = SimpleExecutor::execute(&payload);
    HttpResponse::Ok().json(result)
}

pub async fn run_server() -> std::io::Result<()> {
    let port = get_server_port();
    let server_address = (Ipv4Addr::UNSPECIFIED, port);
    let _swagger_url = format!(
        "http://{}:{}/swagger-ui/",
        server_address.0, server_address.1
    );

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                if let Some(origin_str) = origin.to_str().ok() {
                    origin_str.ends_with(":5173")
                } else {
                    false
                }
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .route("/execute", web::post().to(execute_code))
    })
    .bind(server_address)?
    .run()
    .await
}

fn get_server_port() -> u16 {
    env::var("APP_PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.to_string())
        .parse()
        .unwrap_or(DEFAULT_PORT)
}
