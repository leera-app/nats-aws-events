// main.rs

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudwatchlogs::Client as CloudWatchLogsClient;
use aws_sdk_lambda::Client as LambdaClient;
use lambda_trigger::run_lambda_trigger;
use status_checker::run_status_checker;
use std::env;
use std::error::Error;
use tokio::try_join;


mod lambda_trigger;
mod status_checker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error >> {
    // Load environment variables
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let nats_client = async_nats::connect(&nats_url).await?;
    let nats_client2 = nats_client.clone(); // Clone for the second task

    // Connect to NATS
    println!("✅ Connected to NATS at {}", nats_url);

    // Load AWS config and create Lambda client
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let lambda_client = LambdaClient::new(&config);
    let logs_client = CloudWatchLogsClient::new(&config);
    println!("✅ Initialized AWS Lambda client");
    // Run the lambda trigger loop
    try_join!(
        run_lambda_trigger(nats_client.clone(), lambda_client.clone()),
        run_status_checker(nats_client2, lambda_client,logs_client)
    )?;
    Ok(())
}
