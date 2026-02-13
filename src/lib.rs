//! Wrapper around the crate 'ureq' to make downloading files easier
//! 
//!  Example:
//! ```
//! use udownload::download;
//! use std::path::PathBuf;
//! 
//! let downloaded_file_path: PathBuf = download("https://pokeapi.co/api/v2/pokemon/golbat", PathBuf::from("file.json")).unwrap();
//! ```

use std::{fs::File, path::PathBuf, io::{Read, Write}};

use thiserror::Error;
#[cfg(feature = "progress-bar")]
use indicatif::*;


#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Net(#[from] ureq::Error),
    #[error("Miscelanious Error: {0}")]
    Misc(String),
}

/// Download a file with parameters `url: &str` `save_path: PathBuf` and returns a `Result<PathBuf, Error>`
pub fn download(url: &str, save_path: PathBuf) -> Result<PathBuf, Error> {

    let mut response = ureq::get(url).call()?;
    let mut reader = response.body_mut().as_reader();
    let mut file = File::create(save_path.clone())?;

    let mut buffer = [0u8; 8 * 1024];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
    }

    Ok(save_path)
}

#[cfg(feature = "progress-bar")]
/// Same function as before , but with a progess bar
pub fn download_with_progressbar(url: &str, save_path: PathBuf) -> Result<PathBuf, Error> {
    let mut response = ureq::get(url).call()?;
    let size = response.body().content_length().unwrap_or(0);
    let mut reader = response.body_mut().as_reader();
    let mut file = File::create(save_path.clone())?;

    let pb = ProgressBar::new(size);

    pb.set_style(
        ProgressStyle::default_bar()
        .template(
            "{bar:80.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
        )
        .unwrap()
        .progress_chars("=> "),
    );

    let mut buffer = [0u8; 8 * 1024];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        pb.inc(n as u64);
        file.write_all(&buffer[..n])?;
    }

    Ok(save_path)
}

#[cfg(test)]
mod tests {
    use std::{env::temp_dir, fs::read_to_string};

    use super::*;

    #[test]
    fn test_download() {
        let mut server = mockito::Server::new();

        let url = server.url();

        server.mock("GET", "/hello")
            .with_status(201)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body("world")
            .create();

        let tmpdir = temp_dir();
        let downloaded_file_path: PathBuf = download(&(url + "/hello"), tmpdir.join("file.txt")).unwrap();
        let downloaded_file_contents = read_to_string(downloaded_file_path).unwrap_or("download".to_string());

        let local_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("example.txt");
        let local_file_contents = read_to_string(local_file_path).unwrap_or("local".to_string());

        assert_eq!(downloaded_file_contents, local_file_contents);
    }
}
