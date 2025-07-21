use async_nats::{Client, jetstream,HeaderMap};
use aws_sdk_lambda::Client as LambdaClient;
use futures::StreamExt;
use serde_json::Value;
use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use aws_sdk_cloudwatchlogs::{types::FilteredLogEvent, Client as CloudWatchLogsClient};

pub async fn run_status_checker(client: Client,  lambda_client: LambdaClient,logs_client: CloudWatchLogsClient) -> Result<(), async_nats::Error> {
    // Connect to NATS
    // let client = async_nats::connect(nats_url).await?;
    let js = jetstream::new(client.clone());

    // Create or bind to the stream & consumer
    // let stream = js.get_stream("MY_EVENTS_STREAM").await?;
    let stream = match js.get_stream("status_bridge").await {
        Ok(s) => {
            println!("✅ Stream found");
            s
        }
        Err(e) => {
           // Make sure your function returns `Box<dyn Error>`
            match js.create_stream(jetstream::stream::Config {
                name: "status_bridge".to_string(),
                subjects: vec!["my.status".to_string()],
                ..Default::default()
            }).await {
                Ok(s) => {
                    println!("✅ Successfully created stream");
                    s
                }
                Err(e) => {
                    println!("❌ Failed to create stream: {:?}", e);
                    return Err(Box::new(e)); // Make sure your function returns `Box<dyn Error>`
                }
            }
        }
    };

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("lambda_trigger".into()),
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            ..Default::default()
        })
        .await?;

    let mut messages = consumer.messages().await?;

    while let Some(msg) = messages.next().await {
        let msg = msg?;
        let payload: Value = serde_json::from_slice(&msg.payload)?;

        let retry_index = payload["retry_index"].as_u64().unwrap_or(0);
        let event_id = payload["event_id"].as_str().unwrap_or("");
        let lambda_arn = payload["lambda_arn"].as_str().unwrap();
        let lambda_request_id = payload["lambda_request_id"].as_str().unwrap_or("");

        let lambda_failed = check_lambda_status(&logs_client, lambda_arn, lambda_request_id).await?;


        if lambda_failed && retry_index < 6 {
            let mut retry_payload = payload.clone();
            retry_payload["retry_index"] = (retry_index + 1).into();

            let delay = get_delay_seconds(retry_index + 1);

            let mut headers = async_nats::HeaderMap::new();


            headers.insert("Nats-Delay", format!("{}s", delay));
            headers.insert("Nats-Msg-Id", event_id);

            client
                .publish_with_headers::<String>(
                    "my.event".into(),
                    headers,
                    serde_json::to_vec(&retry_payload)?.into(),
                )
                .await?;
        }

        msg.ack().await?;
    }

    Ok(())
}


async fn check_lambda_status(
    logs_client: &CloudWatchLogsClient,
    lambda_arn: &str,
    request_id: &str,
) -> Result<bool, anyhow::Error> {
    // Extract Lambda function name from ARN
    let function_name = lambda_arn
        .split(':')
        .last()
        .ok_or_else(|| anyhow::anyhow!("Invalid Lambda ARN"))?;

    // Query CloudWatch Logs for the Lambda execution
    let log_group_name = format!("/aws/lambda/{}", function_name);
    let filter_output = logs_client
        .filter_log_events()
        .log_group_name(&log_group_name)
        .filter_pattern(&format!("REPORT RequestId: {}", request_id))
        .send()
        .await?;

    // Check if the log indicates a failure (e.g., by parsing the log events)
    let lambda_failed = log_events_indicate_failure(&filter_output.events.unwrap_or_default());
    Ok(lambda_failed)
}

fn log_events_indicate_failure(events: &[FilteredLogEvent]) -> bool {
    // Implement logic to parse log events and determine if the Lambda failed
    // For example, look for "Task timed out" or "ERROR" in the logs
    events.iter().any(|event| {
        event.message().map(|msg| msg.contains("ERROR") || msg.contains("Task timed out")).unwrap_or(false)
    })
}

fn get_delay_seconds(index: u64) -> u64 {
    let delays = [60, 600, 1800, 3600, 14400, 28800, 86400];
    *delays.get(index as usize).unwrap_or(&86400)
}
