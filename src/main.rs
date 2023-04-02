use image::imageops::FilterType;
use img_comp::Response;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::Deserialize;

#[derive(Deserialize)]
struct Request {
    dir: String,
    scale_op: String,
    scale_factor: u32,
    filter: String,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let root_dir = format!("/mnt/efs/{}", event.payload.dir);
    // scale opts: "up", "down"
    let scale_op = event.payload.scale_op;
    let scale_factor = event.payload.scale_factor;
    // filter opts
    let filter = match event.payload.filter.to_lowercase().as_str() {
        "gauss" => FilterType::Gaussian,
        "near" => FilterType::Nearest,
        "tri" => FilterType::Triangle,
        "cmr" => FilterType::CatmullRom,
        "lcz" => FilterType::Lanczos3,
        _ => FilterType::Gaussian,
    };

    // Match the scaling case and return response
    match scale_op.as_str() {
        "up" => {
            println!(
                "Scaling up by factor {} with filter {:?}",
                scale_factor, filter
            );
            let resp = img_comp::scale_up(root_dir, scale_factor, filter).await?;
            Ok(resp)
        }
        "down" => {
            println!(
                "Scaling down by factor {} with filter {:?}",
                scale_factor, filter
            );
            let resp = img_comp::scale_down(root_dir, scale_factor, filter).await?;
            Ok(resp)
        }
        _ => {
            // Return error
            let resp = Response {
                time: "ERROR incorrect scale_op".to_string(),
                size: "ERROR incorrect scale_op".to_string(),
            };
            Ok(resp)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
