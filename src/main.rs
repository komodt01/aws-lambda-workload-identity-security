use aws_sdk_s3::Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

async fn function_handler(
    _event: LambdaEvent<Value>,
    client: &Client,
) -> Result<Value, Error> {
    let resp = client.list_buckets().send().await?;

    let buckets = resp
        .buckets
        .unwrap_or_default()
        .into_iter()
        .map(|bucket| bucket.name.unwrap_or_default())
        .collect::<Vec<String>>();

    Ok(json!({
        "bucket_count": buckets.len(),
        "buckets": buckets
    }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let handler = service_fn(|event| function_handler(event, &client));

    run(handler).await
}