use std::io::{self, stdout};

use download::Download;
use flate2::read::GzDecoder;
use reqwest::blocking::Client;

mod download;

fn main() -> anyhow::Result<()> {
    let url = String::from("http://localhost:8000/test.gz");
    let client = Client::new();

    let mut stdout = stdout();
    let compressed = Download::from(client, url)?;
    let mut decompressed = GzDecoder::new(compressed);
    let _read = io::copy(&mut decompressed, &mut stdout)?;

    Ok(())
}
