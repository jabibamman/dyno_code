use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Deserialize)]
struct ExecutionPayload {
    language: String,
    code: String,
}

#[derive(Serialize)]
struct ExecutionResult {
    output: String,
    error: String,
}

async fn execute_code(payload: web::Json<ExecutionPayload>) -> impl Responder {
    let output = match payload.language.as_str() {
        "python" => Command::new("python3").arg("-c").arg(&payload.code).output(),
        "lua" => Command::new("lua").arg("-e").arg(&payload.code).output(),
        // TODO: Add more languages
        _ => {
            return HttpResponse::BadRequest().body("Language not supported");
        },
    };

    let result = match output {
        Ok(output) => ExecutionResult {
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(e) => ExecutionResult {
            output: "".to_string(),
            error: e.to_string(),
        },
    };

    HttpResponse::Ok().json(result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/execute", web::post().to(execute_code)))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
