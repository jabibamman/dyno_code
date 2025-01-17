use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{
    http,
    web,
    App,
    HttpResponse,
    HttpServer,
    Responder,
};
use base64::{
    prelude::BASE64_STANDARD,
    Engine,
};
use futures_util::stream::TryStreamExt;
use log::{
    error,
    info,
};

use serde_json::json;

use std::net::Ipv4Addr;
use tokio::{
    fs::File,
    io::AsyncWriteExt,
};
use uuid::Uuid;

use crate::executor::{
    CodeExecutor,
    K8sExecutor,
};

use crate::types::{
    ExecutionPayload,
    ExecutionResult,
};

use crate::api::{
    check_version,
    get_server_port,
    health_check,
};

async fn execute_code(mut payload: Multipart) -> impl Responder {
    let mut language = None;
    let mut code = None;
    let mut output_extension = Some(".txt".to_string());
    let mut input_file_path = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let field_name = match content_disposition.get_name() {
            Some(name) => name.trim().to_string(),
            None => {
                return HttpResponse::BadRequest().body("Missing field name");
            }
        };

        match field_name.as_str() {
            "language" => {
                info!("Received language");
                let mut data = Vec::new();
                while let Some(chunk) = field.try_next().await.unwrap() {
                    data.extend_from_slice(&chunk);
                }
                language = Some(String::from_utf8(data).unwrap());
            }
            "code" => {
                info!("Received code");
                let mut data = Vec::new();
                while let Some(chunk) = field.try_next().await.unwrap() {
                    data.extend_from_slice(&chunk);
                }
                code = Some(String::from_utf8(data).unwrap());
            }
            "output_extension" => {
                info!("Received output extension");
                let mut data = Vec::new();
                while let Some(chunk) = field.try_next().await.unwrap() {
                    data.extend_from_slice(&chunk);
                }
                output_extension = Some(String::from_utf8(data).unwrap());
                if output_extension == Some("".to_string())
                    || output_extension == Some("null".to_string())
                {
                    output_extension = Some(".txt".to_string());
                }
            }
            "input_file" => {
                info!("Received input file");
                let filename = content_disposition
                    .get_filename()
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| Uuid::new_v4().to_string());

                let extension = filename.split('.').last().unwrap_or("");
                let file_path = if extension.is_empty() {
                    format!("/mnt/shared/{}", Uuid::new_v4())
                } else {
                    format!("/mnt/shared/{}.{}", Uuid::new_v4(), extension)
                };
                info!("Writing input file to: {:?}", file_path);
                let mut file = File::create(&file_path).await.unwrap();
                let mut is_empty = true;

                while let Some(chunk) = field.try_next().await.unwrap() {
                    info!("Writing chunk to file");
                    if !chunk.is_empty() {
                        is_empty = false;
                    }
                    if let Err(e) = file.write_all(&chunk).await {
                        error!("Failed to write chunk to file: {:?}", e);
                        return HttpResponse::InternalServerError().body("Failed to write to file");
                    }
                }

                if let Err(e) = file.flush().await {
                    error!("Failed to flush file: {:?}", e);
                    return HttpResponse::InternalServerError().body("Failed to flush file");
                }

                if is_empty {
                    info!("Empty file received, deleting the file.");
                    if let Err(e) = tokio::fs::remove_file(&file_path).await {
                        error!("Failed to delete empty file: {:?}", e);
                    }
                    return HttpResponse::BadRequest().body("Empty file received");
                }

                input_file_path = Some(file_path);

                let ls_output = std::process::Command::new("ls")
                    .arg("-l")
                    .arg("/mnt/shared")
                    .output()
                    .expect("Failed to execute ls command");
                info!("ls -l /mnt/shared: {:?}", ls_output);

                info!("Input file path: {:?}", input_file_path);
            }
            _ => (),
        }
    }

    info!("Final language: {:?}", language);
    info!("Final code: {:?}", code);
    info!("Final input file path: {:?}", input_file_path);
    info!("Final output extension: {:?}", output_extension);

    match (language.is_none(), code.is_none()) {
        (true, true) => {
            return HttpResponse::BadRequest()
                .body("Missing required fields: 'language' and 'code'");
        }
        (true, false) => {
            return HttpResponse::BadRequest().body("Missing required field: 'language'");
        }
        (false, true) => {
            return HttpResponse::BadRequest().body("Missing required field: 'code'");
        }
        _ => (),
    }

    let payload = ExecutionPayload {
        language: language.unwrap(),
        code: code.unwrap(),
        input_file_path,
        output_extension: output_extension.unwrap(),
    };

    info!("Received request to execute code: {:?}", payload);

    let result = K8sExecutor::execute(&payload).await;
    match result {
        Ok(mut execution_result) => {
            if let Some(ref path) = payload.input_file_path {
                tokio::fs::remove_file(path).await.unwrap();
            }

            if let Some(output_file_path) = execution_result.output_file_path.clone() {
                if !output_file_path.is_empty() {
                    info!(
                        "Successfully processing output with file: {:?}",
                        execution_result
                    );

                    let file_content = tokio::fs::read(&output_file_path)
                        .await
                        .unwrap_or_else(|_| Vec::new());

                    execution_result.output_file_content =
                        Some(BASE64_STANDARD.encode(&file_content));
                }
            }

            let json_response = json!({
                "output": execution_result.output,
                "error": execution_result.error,
                "output_file_path": execution_result.output_file_path,
                "output_file_content": execution_result.output_file_content,
            });

            if !execution_result.error.is_empty() {
                HttpResponse::BadRequest().json(json_response)
            } else {
                info!("Successfully returning output: {:?}", execution_result);
                HttpResponse::Ok().json(json_response)
            }
        }
        Err(e) => {
            if let Some(ref path) = payload.input_file_path {
                tokio::fs::remove_file(path).await.unwrap();
            }
            error!("Error executing code: {:?}", e);
            HttpResponse::InternalServerError().json(ExecutionResult {
                output: "".to_string(),
                error: e.to_string(),
                output_file_path: None,
                output_file_content: None,
            })
        }
    }
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
                    origin_str.ends_with(":5173") || origin_str.contains("code-valley.xyz")
                } else {
                    false
                }
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .route("/execute", web::post().to(execute_code))
            .route("/health", web::get().to(health_check))
            .route("/version", web::get().to(check_version))
    })
    .bind(server_address)?
    .run()
    .await
}
