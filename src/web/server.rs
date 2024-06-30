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
use futures_util::stream::TryStreamExt;
use log::{
    error,
    info,
};

use std::{
    env,
    net::Ipv4Addr,
};
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
            "input_file" => {
                info!("Received input file");
                let file_path = format!("/mnt/shared/{}", Uuid::new_v4());
                info!("Writing input file to: {:?}", file_path);
                let mut file = File::create(&file_path).await.unwrap();

                while let Some(chunk) = field.try_next().await.unwrap() {
                    info!("Writing chunk to file");
                    if let Err(e) = file.write_all(&chunk).await {
                        error!("Failed to write chunk to file: {:?}", e);
                        return HttpResponse::InternalServerError().body("Failed to write to file");
                    }
                }

                if let Err(e) = file.flush().await {
                    error!("Failed to flush file: {:?}", e);
                    return HttpResponse::InternalServerError().body("Failed to flush file");
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

    if language.is_none() || code.is_none() {
        return HttpResponse::BadRequest().body("Missing required fields: 'language' and 'code'");
    }

    let payload = ExecutionPayload {
        language: language.unwrap(),
        code: code.unwrap(),
        input_file_path,
    };

    info!("Received request to execute code: {:?}", payload);

    let result = K8sExecutor::execute(&payload).await;
    match result {
        Ok(execution_result) => {
            if !execution_result.error.is_empty() {
                if let Some(ref path) = payload.input_file_path {
                    tokio::fs::remove_file(path).await.unwrap();
                }
                HttpResponse::BadRequest().json(execution_result)
            } else {
                if let Some(ref path) = payload.input_file_path {
                    tokio::fs::remove_file(path).await.unwrap();
                }
                info!("Successfully executed code: {:?}", execution_result);
                HttpResponse::Ok().json(execution_result)
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
