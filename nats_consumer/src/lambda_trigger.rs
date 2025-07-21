// lambda_trigger.rs
use async_nats::{jetstream, Client};
use aws_sdk_lambda::primitives::Blob;
use serde_json::Value;
use futures::StreamExt;
use anyhow::Result;

pub async fn run_lambda_trigger(client: Client, lambda_client: aws_sdk_lambda::Client) ->Result<(), Box<dyn std::error::Error>> {
    // Connect to NATS
    // let client = async_nats::connect(nats_url).await?;
    let js = jetstream::new(client.clone());

    let stream = match js.get_stream("my_bridge").await {
        Ok(s) => {
            println!("✅ Stream found");
            s
        }
        Err(_e) => {
           // Make sure your function returns `Box<dyn Error>`
            match js.create_stream(jetstream::stream::Config {
                name: "my_bridge".to_string(),
                subjects: vec!["my.event".to_string()],
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

        let lambda_arn = payload["lambda_arn"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing lambda_arn"))?;
        let retry_index = payload
            .get("retry_index")
            .and_then(Value::as_u64)
            .unwrap_or(0);

        // Invoke the Lambda function
        let invoke_result = lambda_client
            .invoke()
            .invocation_type("Event".into())
            .function_name(lambda_arn)
            .payload(Blob::new(serde_json::to_vec(&payload)?))
            .send()
            .await?;

        let request_id = invoke_result
            .executed_version()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown".to_string()); // Or handle as needed

        // Prepare status check payload
        let mut status_payload = payload.clone();
        status_payload["retry_index"] = retry_index.into();
        status_payload["event_id"] = msg.info().map(|i| i.stream_sequence.to_string()).unwrap_or_default().into();
        status_payload["lambda_request_id"] = request_id.into(); // Add RequestId to payload

        // Delay header
        let delay_secs = get_delay_seconds(retry_index);
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Nats-Delay", format!("{}s", delay_secs));

        // Publish delayed status event
        client
            .publish_with_headers::<String>(
                "check.lambda.status".into(),
                headers,
                serde_json::to_vec(&status_payload)?.into(),
            )
            .await?;

        match msg.ack().await {
            Ok(_) => {},
            Err(e) => return Ok(()),
        };
    }

    Ok(())
}

fn get_delay_seconds(idx: u64) -> u64 {
    let delays = [60, 600, 1800, 3600, 14400, 28800, 86400];
    *delays.get(idx as usize).unwrap_or(&86400)
}
