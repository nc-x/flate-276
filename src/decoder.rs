use std::{
    cell::RefCell,
    io::{BufReader, Read},
    rc::Rc,
};

use flate2::bufread::MultiGzDecoder;
use reqwest::blocking::Response;

#[derive(Clone)]
pub struct Reader(Rc<RefCell<Response>>);

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().read(buf)
    }
}

pub struct GzipDecoder {
    inner: Reader,
    decoder: MultiGzDecoder<BufReader<Reader>>,
}

impl GzipDecoder {
    pub fn from(r: Response) -> Self {
        let inner = Reader(Rc::new(RefCell::new(r)));
        Self {
            inner: inner.clone(),
            decoder: MultiGzDecoder::new(BufReader::new(inner)),
        }
    }

    pub fn replace(&self, r: Response) {
        self.inner.0.as_ref().replace(r);
    }
}

impl Read for GzipDecoder {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.decoder.read(buf)
    }
}
