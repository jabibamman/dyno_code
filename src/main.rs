use dotenv::dotenv;
use dyno_code::web::run_server;
use log::info;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting server...");
    run_server().await
}
