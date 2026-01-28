//! Wrapper around the crate 'ureq' to make downloading files easier
//! 
//!
//!
//! 
//! 

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
        let tmpdir = temp_dir();
        let downloaded_file_path: PathBuf = download("https://pokeapi.co/api/v2/pokemon/golbat", tmpdir.join("file.json")).unwrap_or(PathBuf::new());
        let downloaded_file_contents = read_to_string(downloaded_file_path).unwrap_or("download".to_string());

        let local_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("example.json");
        let local_file_contents = read_to_string(local_file_path).unwrap_or("local".to_string());

        assert_eq!(downloaded_file_contents, local_file_contents);
    }
}
