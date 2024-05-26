use crate::executor::CodeExecutor;
use crate::types::{ExecutionPayload, ExecutionResult};
use k8s_openapi::api::batch::v1::Job;
use kube::{api::PostParams, Api, Client};
use log::{debug, info};
use serde_json::json;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

pub struct K8sExecutor;

#[async_trait::async_trait]
impl CodeExecutor for K8sExecutor {
    async fn execute(
        payload: &ExecutionPayload,
    ) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
        let client = Client::try_default().await?;
        let project_id = env::var("GOOGLE_CLOUD_PROJECT_ID")
            .expect("GOOGLE_CLOUD_PROJECT_ID environment variable must be set");
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
                            "image": format!("gcr.io/{}/executor:latest", project_id),
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
        let logs = Self::wait_for_pod_and_get_logs(&client, &job_name).await?;

        Ok(ExecutionResult {
            output: logs,
            error: String::new(),
        })
    }
}

impl K8sExecutor {
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
}
