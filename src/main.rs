use std::io::{self, stdout};

use download::Download;
use reqwest::blocking::Client;

mod decoder;
mod download;

fn main() -> anyhow::Result<()> {
    let url = String::from("http://localhost:8000/test.gz");
    let client = Client::new();

    let mut stdout = stdout();
    let mut downloader = Download::from(client, url)?;
    let read = io::copy(&mut downloader, &mut stdout)?;
    assert_eq!(downloader.content_length, read);

    Ok(())
}
