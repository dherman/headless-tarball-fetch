use reqwest::header::ACCEPT_RANGES;
use std::io::{self, Read};
use reqwest::header::RANGE;
use reqwest::header::CONTENT_LENGTH;
use reqwest::{get, Response};

/// Determines the length of an HTTP response's content in bytes, using
/// the HTTP `"Content-Length"` header.
fn content_length(response: &Response) -> u64 {
    match response.headers().get(CONTENT_LENGTH) {
        Some(content_length) => content_length.to_str().unwrap().parse().unwrap(),
        None => {
            panic!("missing header: Content-Length");
        }
    }
}

// From http://www.gzip.org/zlib/rfc-gzip.html#member-format
//
//   0   1   2   3   4   5   6   7
// +---+---+---+---+---+---+---+---+
// |     CRC32     |     ISIZE     |
// +---+---+---+---+---+---+---+---+
//
// ISIZE (Input SIZE)
//    This contains the size of the original (uncompressed) input data modulo 2^32.

/// Unpacks the `isize` field from a gzip payload as a 64-bit integer.
fn unpack_isize(packed: [u8; 4]) -> u64 {
    let unpacked32: u32 =
        ((packed[0] as u32)      ) +
        ((packed[1] as u32) <<  8) +
        ((packed[2] as u32) << 16) +
        ((packed[3] as u32) << 24);

    unpacked32 as u64
}

/// Fetches just the `isize` field (the field that indicates the uncompressed size)
/// of a gzip file from a URL. This makes two round-trips to the server but avoids
/// downloading the entire gzip file. For very small files it's unlikely to be
/// more efficient than simply downloading the entire file up front.
fn fetch_isize(url: &str, len: u64) -> Result<[u8; 4], reqwest::Error> {
    let client = reqwest::Client::new();
    let mut response = client.get(url)
        .header(RANGE, &format!("bytes={}-{}", len - 4, len - 1)[..])
        .send()?;

    if !response.status().is_success() {
        panic!("http error: {}", response.status());
    }

    let actual_length = content_length(&response);

    if actual_length != 4 {
        panic!("unexpected content length: expected 4 bytes, got {}", actual_length);
    }

    let mut buf = [0; 4];
    response.read_exact(&mut buf).unwrap();
    Ok(buf)
}

/// Encapsulates a connection to fetch a tarball from a remote server.
///
/// Supports computing the compressed size and optionally, if the remote
/// server supports byte range requests, the uncompressed size.
///
/// Implements `Read` for lazy reading from the connection.
pub struct RemoteTarball {
    url: String,
    response: Response,
}

impl RemoteTarball {
    /// Constructor for a remote tarball. Initiates the fetch of the full payload.
    pub fn fetch(url: &str) -> reqwest::Result<Self> {
        let response = get(url)?;
        Ok(Self {
            url: url.to_string(),
            response,
        })
    }

    /// Determines the size of the full payload from the headers.
    pub fn compressed_size(&self) -> u64 {
        match self.response.headers().get(CONTENT_LENGTH) {
            Some(content_length) => content_length.to_str().unwrap().parse().unwrap(),
            None => {
                panic!("missing header: Content-Length");
            }
        }
    }

    /// Checks the headers to see if we can send a separate request to compute
    /// the uncompressed size as well.
    fn supports_byte_ranges(&self) -> bool {
        if let Some(value) = self.response.headers().get(ACCEPT_RANGES) {
            if let Some(s) = value.to_str().ok() {
                return s.trim() == "bytes";
            }
        }
        return false;
    }

    /// Computes the uncompressed size by making a separate concurrent request
    /// to the server, if supported.
    pub fn uncompressed_size(&self) -> reqwest::Result<Option<u64>> {
        if !self.supports_byte_ranges() {
            return Ok(None);
        }

        let packed = fetch_isize(&self.url[..], self.compressed_size())?;
        Ok(Some(unpack_isize(packed)))
    }
}

impl Read for RemoteTarball {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.response.read(buf)
    }
}
