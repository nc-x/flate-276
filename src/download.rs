use regex::Regex;
use reqwest::{
    blocking::Client,
    header::{self, HeaderValue},
};
use std::{
    cmp,
    io::{Cursor, Read},
    str::FromStr,
};

use crate::decoder::GzipDecoder;

const MAX_CHUNK_SIZE: u64 = 1 * 1024 * 1024; // 1 MB

enum State {
    Init,
    NeedData,
    HaveData,
    Done,
}

pub struct Download {
    client: Client,
    pub url: String,
    range_start: u64,
    pub content_length: u64,
    decoder: GzipDecoder,
    buffer: Vec<u8>,
    index: usize,
    state: State,
}

impl Download {
    pub fn from(client: Client, url: String) -> anyhow::Result<Self> {
        // we need an initial range request here because flate2 GzDecoder parses the header in ::new(..) itself
        let response = client
            .get(&url)
            .header(
                header::RANGE,
                HeaderValue::from_str(&format!("bytes=0-1023"))?,
            )
            .send()?
            .error_for_status()?;

        // parse content length from the range header because the content length header would itself give the length of the range response
        let content_length = Self::parse_content_length_from_range_header(&response)?;
        assert!(content_length > 0);

        let decoder = GzipDecoder::from(response);

        Ok(Self {
            client,
            url,
            range_start: 1024,
            content_length,
            decoder,
            buffer: Vec::new(), //
            index: 0,
            state: State::Init,
        })
    }

    fn parse_content_length_from_range_header(
        response: &reqwest::blocking::Response,
    ) -> anyhow::Result<u64> {
        let range = response
            .headers()
            .get(header::CONTENT_RANGE)
            .unwrap()
            .to_str()?;

        let re = Regex::new(r"bytes (\d+)-(\d+)/(\d+)")?;
        let captures = re.captures(range).unwrap();
        let content_length = u64::from_str(&captures[3])?;

        Ok(content_length)
    }
}

impl Read for Download {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match self.state {
                State::Init => {
                    println!(
                        r"
                        Trying to decode the partial gzip from the initial request.
                        This will fail because we have no idea what the actual length of the decoded content is
                        and read_to_end simply tries going past the end and fails because the GzDecoder obviously
                        thinks that this is a corrupt file.
                        "
                    );
                    // Note: Try remoing the `?` to see it go into infinite loop when we replace the reader below and try reading from it.
                    self.decoder.read_to_end(&mut self.buffer)?;
                    println!("Read Successfully");
                    self.state = State::HaveData;
                }
                State::NeedData => {
                    let chunk_size =
                        cmp::min(MAX_CHUNK_SIZE, self.content_length - self.range_start + 1);
                    let range_end = self.range_start + chunk_size - 1;

                    let response = self
                        .client
                        .get(&self.url)
                        .header(
                            header::RANGE,
                            HeaderValue::from_str(&format!(
                                "bytes={}-{}",
                                self.range_start, range_end
                            ))
                            .unwrap(),
                        )
                        .send()
                        .unwrap()
                        .error_for_status()
                        .unwrap();
                    self.range_start = range_end + 1;
                    self.decoder.replace(response); // SWAPPING THE UNDERLYING READER
                    self.decoder.read_to_end(&mut self.buffer)?;
                    self.state = State::HaveData;
                }
                State::HaveData => {
                    let read = Cursor::new(&self.buffer[self.index..]).read(buf)?;
                    self.index += read;
                    if self.index == self.buffer.len() {
                        if self.range_start > self.content_length {
                            self.state = State::Done;
                        } else {
                            self.state = State::NeedData;
                        }
                    }
                    return Ok(read);
                }
                State::Done => return Ok(0),
            }
        }
    }
}
