use actix_cors::Cors;
use actix_web::http;
use actix_web::{web, App, HttpResponse, HttpServer};
use k8s_openapi::api::batch::v1::Job;
use kube::{api::PostParams, Api, Client};
use log::{debug, error, info};
use serde_json::json;
use std::env;
use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::time::sleep;

use crate::types::{ExecutionPayload, ExecutionResult};

const DEFAULT_PORT: u16 = 8080;

async fn execute_code(payload: web::Json<ExecutionPayload>) -> impl actix_web::Responder {
    info!("Received request to execute code: {:?}", payload);

    let result = create_k8s_job(&payload).await;
    match result {
        Ok(logs) => {
            info!("Job executed successfully. Logs: {}", logs);
            HttpResponse::Ok().json(ExecutionResult {
                output: logs,
                error: String::new(),
            })
        }
        Err(e) => {
            error!("Job execution failed: {}", e);
            HttpResponse::InternalServerError().json(ExecutionResult {
                output: String::new(),
                error: e.to_string(),
            })
        }
    }
}
async fn create_k8s_job(payload: &ExecutionPayload) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    let jobs: Api<Job> = Api::default_namespaced(client.clone());

    let job_name = format!("job-{}", uuid::Uuid::new_v4());
    info!("Creating Job with name: {}", job_name);

    let job_spec = json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": job_name
        },
        "spec": {
            "template": {
                "metadata": {
                    "name": job_name
                },
                "spec": {
                    "containers": [{
                        "name": "executor",
                        "image": "gcr.io/pa2024-421814/executor:latest",
                        "command": ["sh", "-c", format!("./executor_script.sh {} '{}'", payload.language, payload.code)],
                    }],
                    "restartPolicy": "Never"
                }
            },
            "backoffLimit": 4
        }
    });

    let job_spec: Job = serde_json::from_value(job_spec)?;
    jobs.create(&PostParams::default(), &job_spec).await?;
    let logs = wait_for_pod_and_get_logs(&client, &job_name).await?;
    Ok(logs)
}

async fn wait_for_pod_and_get_logs(
    client: &Client,
    job_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::default_namespaced(client.clone());

    for _ in 0..10 {
        let pod_list = pods
            .list(&Default::default())
            .await?
            .into_iter()
            .filter(|pod| {
                pod.metadata
                    .labels
                    .as_ref()
                    .and_then(|labels| labels.get("job-name"))
                    .map_or(false, |name| name == job_name)
            })
            .collect::<Vec<_>>();

        if let Some(pod) = pod_list.first() {
            let pod_name = pod.metadata.name.as_ref().ok_or("Pod name not found")?;
            let log_params = Default::default();
            match pods.logs(pod_name, &log_params).await {
                Ok(logs) => return Ok(logs),
                Err(_) => {
                    debug!("Pod is not ready yet, retrying...");
                    sleep(Duration::from_secs(1)).await;
                }
            }
        } else {
            debug!("No pods found for the job, retrying...");
            sleep(Duration::from_secs(1)).await;
        }
    }

    Err("No pods found for the job".into())
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
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
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
