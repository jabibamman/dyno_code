use crate::executor::SimpleExecutor;
use crate::types::ExecutionPayload;
use actix_web::{web, App, HttpResponse, HttpServer};

async fn execute_code(payload: web::Json<ExecutionPayload>) -> impl actix_web::Responder {
    let result = SimpleExecutor::execute(&payload);
    HttpResponse::Ok().json(result)
}

pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/execute", web::post().to(execute_code)))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
