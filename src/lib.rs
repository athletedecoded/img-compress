use image::{
    imageops::FilterType,
    ImageFormat
};
use std::{
    fmt, 
    fs::{File, read_dir, metadata},
    time::{Duration, Instant}
};

use lambda_runtime::Error;
use rayon::prelude::*;
use serde::Serialize;

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

// Function to scale down all images in /mnt/efs
pub async fn scale_down(files: Vec<String>, size: u32, filter: FilterType) -> Result<Response, Error> {
    // Make subdir ./scaled-{size} if DNE
    let subdir = format!("/mnt/efs/scaled-{}", size);
    if !std::path::Path::new(&subdir).exists() {
        std::fs::create_dir(&subdir).unwrap();
    }
    // Start the clock
    let timer = Instant::now();
    // Parallelize the scaling of the images
    files.par_iter().for_each(|fpath| {
        // extract filename from file path
        let fname = fpath.split('/').last().unwrap();
        let img = image::open(fpath).unwrap();
        let scaled = img.resize(size, size, filter);
        let mut output = File::create(format!("{}/{}", &subdir, &fname)).unwrap();
        scaled.write_to(&mut output, ImageFormat::Png).unwrap();
    });
    let resp_msg = format!("Scale down took {}", Elapsed::from(&timer));
    // Check size of scaled images
    let new_size = walk_efs(&subdir).await.unwrap().size;
    // Response
    let resp = Response {
        time: resp_msg,
        size: new_size,
    };
    Ok(resp)
}