use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use image::imageops::FilterType;
use serde::Deserialize;
use img_comp::Response;

#[derive(Deserialize)]
struct Request {
    scale: String,
    filter: String,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // scale opts: "up", "down"
    let scale = event.payload.scale;
    // filter opts
    let filter = match event.payload.filter.as_str() {
        "Nearest" => FilterType::Nearest,
        "Triangle" => FilterType::Triangle,
        "CatmullRom" => FilterType::CatmullRom,
        "Lanczos3" => FilterType::Lanczos3,
        _ => FilterType::Gaussian,
    };
    // Walk efs
    let walk = img_comp::walk_efs("/mnt/efs").await?;
    let files = walk.files;
    let init_size = walk.size;

    // Match the scaling case and return response
    match scale.as_str() {
        // "up" => {
        //     img_comp::scale_up(files, 2, filter).await;
        // }
        "down" => {
            let resp = img_comp::scale_down(files, 200, filter).await?;
            Ok(resp)
        }
        _ => {
            println!("Invalid scale option");
            Ok(Response {
                time: "ERROR: 0".to_string(),
                size: init_size,
            })
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
