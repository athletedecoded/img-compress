use image::{imageops::FilterType, ImageFormat};
use lambda_runtime::Error;
use rayon::prelude::*;
use serde::Serialize;
use std::{
    fmt,
    fs::{metadata, read_dir, OpenOptions},
    time::{Duration, Instant},
};

struct Elapsed(Duration);

impl Elapsed {
    fn from(start: &Instant) -> Self {
        Elapsed(start.elapsed())
    }
}

#[derive(Serialize)]
pub struct DirData {
    pub files: Vec<String>,
    pub size: String,
}

#[derive(Serialize)]
pub struct Response {
    pub time: String,
    pub size: String,
}

impl fmt::Display for Elapsed {
    fn fmt(&self, out: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match (self.0.as_secs(), self.0.subsec_nanos()) {
            (0, n) if n < 1000 => write!(out, "{} ns", n),
            (0, n) if n < 1000_000 => write!(out, "{} µs", n / 1000),
            (0, n) => write!(out, "{} ms", n / 1000_000),
            (s, n) if s < 10 => write!(out, "{}.{:02} s", s, n / 10_000_000),
            (s, _) => write!(out, "{} s", s),
        }
    }
}

// Helper function to list all files in the /mnt/efs directory
pub async fn walk_efs(dir_path: &str) -> Result<DirData, Error> {
    // Track total size and num of files
    let mut total_size = 0;
    // List all files in the directory
    println!("Collecting filepaths in {}", dir_path);
    let mut files = Vec::new();
    for entry in read_dir(dir_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            files.push(path.display().to_string());
        }
        // Get entry size
        let metadata = metadata(path).unwrap();
        let size = metadata.len();
        total_size += size;
    }
    // Response
    let resp = DirData {
        files,
        size: format!("{total_size} bytes"),
    };
    Ok(resp)
}

// Function to scale down all images in /mnt/efs by scale_factor
pub async fn scale_down(
    root_dir: String,
    scale_factor: u32,
    filter: FilterType,
) -> Result<Response, Error> {
    // Walk efs
    let walk = walk_efs(&root_dir).await?;
    let files = walk.files;
    let init_size = walk.size;

    // Start the clock
    let timer = Instant::now();

    // Parallelize the scaling of the images
    println!("Running parallel down scaling of images...");
    files.par_iter().for_each(|fpath| {
        // extract filename from file path
        let img = image::open(fpath).unwrap();
        let format = ImageFormat::from_path(fpath).unwrap();
        let (width, height) = image::image_dimensions(fpath).unwrap();
        let scaled = img.resize(width / scale_factor, height / scale_factor, filter);
        // Overwrite image file
        let mut output = OpenOptions::new().write(true).truncate(true).open(fpath).unwrap();
        scaled.write_to(&mut output, format).unwrap();
    });
    // Check size of scaled images
    let new_size = walk_efs(&root_dir).await.unwrap().size;
    // Response
    let resp = Response {
        time: format!("Scale down took {}", Elapsed::from(&timer)),
        size: format!(
            "Init dir size: {} --> Scaled dir size: {} .",
            init_size, new_size
        ),
    };
    Ok(resp)
}

// Function to scale up all images in /mnt/efs by scale_factor
pub async fn scale_up(
    root_dir: String,
    scale_factor: u32,
    filter: FilterType,
) -> Result<Response, Error> {
    // Walk efs
    let walk = walk_efs(&root_dir).await?;
    let files = walk.files;
    let init_size = walk.size;

    // Start the clock
    let timer = Instant::now();

    // Parallelize the scaling of the images
    println!("Running parallel up scaling of images...");
    files.par_iter().for_each(|fpath| {
        // extract filename from file path
        let img = image::open(fpath).unwrap();
        let format = ImageFormat::from_path(fpath).unwrap();
        let (width, height) = image::image_dimensions(fpath).unwrap();
        let scaled = img.resize(width * scale_factor, height * scale_factor, filter);
        // Overwrite image file
        let mut output = OpenOptions::new().write(true).truncate(true).open(fpath).unwrap();
        scaled.write_to(&mut output, format).unwrap();
    });
    // Check size of scaled images
    let new_size = walk_efs(&root_dir).await.unwrap().size;
    // Response
    let resp = Response {
        time: format!("Scale up took {}", Elapsed::from(&timer)),
        size: format!(
            "Init dir size: {} --> Scaled dir size: {} .",
            init_size, new_size
        ),
    };
    Ok(resp)
}
