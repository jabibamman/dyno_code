use crate::executor::CodeExecutor;
use crate::types::{ExecutionPayload, ExecutionResult};
use k8s_openapi::api::batch::v1::{Job, JobStatus};
use kube::{
    api::{DeleteParams, PostParams},
    Api, Client,
};
use log::{debug, error, info};
use serde_json::json;
use std::env;
use std::time::Duration;
use tokio::task;
use tokio::time::sleep;

pub struct K8sExecutor;
const DEFAULT_ERROR_MESSAGE: &str = "EXECUTOR_ERROR";

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
                "backoffLimit": 2
            }
        });

        let job_spec: Job = serde_json::from_value(job_spec)?;
        jobs.create(&PostParams::default(), &job_spec).await?;
        let (output, error) = Self::wait_for_pod_and_get_logs(&client, &job_name).await?;

        let jobs_clone = jobs.clone();
        let job_name_clone = job_name.clone();
        task::spawn(async move {
            if let Err(e) = Self::cleanup_job(&jobs_clone, &job_name_clone).await {
                error!("Failed to clean up job {}: {:?}", job_name_clone, e);
            }
        });

        Ok(ExecutionResult {
            output,
            error,
        })
    }
}

impl K8sExecutor {
    async fn wait_for_pod_and_get_logs(
        client: &Client,
        job_name: &str,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::default_namespaced(client.clone());

        for _ in 0..180 {
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
                    Ok(logs) => {
                        if logs.contains(DEFAULT_ERROR_MESSAGE) {
                            return Ok((String::new(), logs.replace(DEFAULT_ERROR_MESSAGE, "").trim().to_string()));
                        }
                        
                        return Ok((logs.trim().to_string(), String::new()));  
                    }
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

    async fn cleanup_job(
        jobs: &Api<Job>,
        job_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for _ in 0..60 {
            let job = jobs.get(job_name).await?;
            if let Some(JobStatus { conditions, .. }) = job.status {
                if let Some(condition) = conditions.as_ref().and_then(|conds| {
                    conds
                        .iter()
                        .find(|&c| c.type_ == "Complete" || c.type_ == "Failed")
                }) {
                    if condition.status == "True" || condition.status == "False" || condition.status == "Unknown" {
                        jobs.delete(job_name, &DeleteParams::default()).await?;
                        info!("Deleted Job with name: {}", job_name);
                        return Ok(());
                    }
                }
            }
            debug!("Job not finished yet, retrying...");
            sleep(Duration::from_secs(1)).await;
        }

        Err("Job did not finish in the expected time".into())
    }
}
