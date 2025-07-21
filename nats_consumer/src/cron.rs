use async_nats::{Client, jetstream::{self, consumer::pull::Messages}, HeaderMap};
use aws_sdk_lambda::Client as LambdaClient;
use serde_json::Value;
use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

pub async fn run_status_checker(nats_client: Client, lambda_client: LambdaClient) -> Result<()> {
    // Setup JetStream context
    let js = jetstream::new(nats_client);

    // Ensure consumer exists or bind to existing one
    let stream_name = "MY_STREAM";
    let consumer_name = "status_checker";

    let consumer = js
        .get_stream(stream_name)
        .await?
        .get_consumer(consumer_name)
        .await?;

    let mut messages: Messages = consumer.messages().await?;

    while let Some(msg) = messages.next().await {
        let payload: Value = serde_json::from_slice(&msg.payload)?;

        let retry_index = payload["retry_index"].as_u64().unwrap_or(0);
        let event_id = payload["event_id"].as_str().unwrap_or("");
        let lambda_arn = payload["lambda_arn"].as_str().unwrap();

        // [SIMPLIFIED] Check Lambda logs or state externally – here we assume failed
        let lambda_failed = true;

        if lambda_failed && retry_index < 6 {
            let mut retry_payload = payload.clone();
            retry_payload["retry_index"] = (retry_index + 1).into();

            let delay = get_delay_seconds(retry_index + 1);

            let mut headers = HeaderMap::new();
            headers.insert("Nats-Delay".into(), format!("{}s", delay).into());
            headers.insert("Nats-Msg-Id".into(), event_id.into());

            js.publish_with_headers("my.event".into(), headers, serde_json::to_vec(&retry_payload)?)
                .await?;
        }

        msg.ack().await?;
    }

    Ok(())
}

fn get_delay_seconds(index: u64) -> u64 {
    let delays = [60, 600, 1800, 3600, 14400, 28800, 86400];
    *delays.get(index as usize).unwrap_or(&86400)
}
