// main.rs

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudwatchlogs::Client as CloudWatchLogsClient;
use aws_sdk_lambda::Client as LambdaClient;
use lambda_trigger::run_lambda_trigger;
use sled::Db;
use status_checker::run_status_checker;
use utils::get_sqlite_path;
use std::env;
use std::error::Error;
use tokio::try_join;


mod lambda_trigger;
mod utils;
mod status_checker;
use aws_credential_types::Credentials;
use aws_types::region::Region;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error >> {
    // Load environment variables
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let nats_client = async_nats::connect(&nats_url).await?;
    let nats_client2 = nats_client.clone(); // Clone for the second task

    // Connect to NATS
    println!("✅ Connected to NATS at {}", nats_url);
    let db_path = get_sqlite_path().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?; // Shared DB path with lambda_trigger service
    let db: Db = sled::open(db_path)?;

    
    // Load AWS credentials from DB
    let access_key_bytes = db.get("aws_access_key")?
        .ok_or_else(|| anyhow::anyhow!("missing aws_access_key"))?;
    let secret_key_bytes = db.get("aws_secret_key")?
        .ok_or_else(|| anyhow::anyhow!("missing aws_secret_key"))?;
    let region_bytes = db.get("aws_region")?
        .ok_or_else(|| anyhow::anyhow!("missing aws_region"))?;

    let access_key = String::from_utf8(access_key_bytes.to_vec())?;
    let secret_key = String::from_utf8(secret_key_bytes.to_vec())?;
    let region_str = String::from_utf8(region_bytes.to_vec())?;


    let credentials = Credentials::new(access_key, secret_key, None, None, "static");
    let config = aws_config::ConfigLoader::default()
        .credentials_provider(credentials)
        .region(Region::new(region_str))
        .load()
        .await;

    let lambda_client = aws_sdk_lambda::Client::new(&config);

    // Load AWS config and create Lambda client

    let logs_client = CloudWatchLogsClient::new(&config);
    println!("✅ Initialized AWS Lambda client");
    // Run the lambda trigger loop
    try_join!(
        run_lambda_trigger(nats_client.clone(), lambda_client.clone(),db.clone()),
        run_status_checker(nats_client2, lambda_client,logs_client)
    )?;
    Ok(())
}
