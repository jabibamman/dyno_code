use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::io::Write;

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
        "rust" => {
            let mut temp_file = tempfile::NamedTempFile::new().unwrap();            
            writeln!(temp_file, "{}", payload.code).unwrap();

            let crate_name = "temp_crate";
            let binary_path = temp_file.path().with_extension("exe");
            let compile_output = Command::new("rustc")
                .arg(temp_file.path())
                .arg("-o")
                .arg(&binary_path)
                .arg("--crate-name")
                .arg(crate_name)
                .output();

            if let Ok(compile_out) = compile_output {
                if compile_out.status.success() {
                    Command::new(binary_path).output()
                } else {
                    return HttpResponse::Ok().json(ExecutionResult {
                        output: "".to_string(),
                        error: String::from_utf8_lossy(&compile_out.stderr).to_string(),
                    });
                }
            } else {
                return HttpResponse::BadRequest().body("Failed to compile Rust code");
            }
        }
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
