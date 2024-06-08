use regex::Regex;
use reqwest::{
    blocking::{Client, Response},
    header::{self, HeaderValue},
};
use std::{cmp, io::Read, str::FromStr};

const MAX_CHUNK_SIZE: u64 = 1 * 1024 * 1024; // 1 MB

pub struct Download {
    client: Client,
    pub url: String,
    range_start: u64,
    pub content_length: u64,
    response: Response,
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

        Ok(Self {
            client,
            url,
            range_start: 1024,
            content_length,
            response,
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
        let mut bytes = self.response.read(buf)?;
        if bytes == 0 && self.range_start < self.content_length {
            let chunk_size = cmp::min(MAX_CHUNK_SIZE, self.content_length - self.range_start + 1);
            let range_end = self.range_start + chunk_size - 1;

            self.response = self
                .client
                .get(&self.url)
                .header(
                    header::RANGE,
                    HeaderValue::from_str(&format!("bytes={}-{}", self.range_start, range_end))
                        .unwrap(),
                )
                .send()
                .unwrap()
                .error_for_status()
                .unwrap();
            self.range_start = range_end + 1;
            bytes = self.response.read(buf)?;
        }
        Ok(bytes)
    }
}
