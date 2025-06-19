use std::io::{self, Write};

use flate2::{write::{GzEncoder, ZlibEncoder}, Compression};

#[derive(Clone)]
pub enum CompressionType {
    None,
    Gzip,
    Zlib,
    Brotli,
    Zstd,
    LZMA,
}


pub trait Compressor {
    fn compress(data: &[u8]) -> io::Result<Vec<u8>>;
}

impl Compressor for GzEncoder<Vec<u8>> {
    fn compress(data: &[u8]) -> io::Result<Vec<u8>> {
        let mut encoder = Self::new(Vec::new(), Compression::new(6));
        encoder.write_all(&data)?;
        encoder.finish()
    }
}

impl Compressor for ZlibEncoder<Vec<u8>> {
    fn compress(data: &[u8]) -> io::Result<Vec<u8>> {
        let mut encoder = Self::new(Vec::new(), Compression::new(6));
        encoder.write_all(&data)?;
        encoder.finish()
    }
}

// implementation of Compressor for Brotli
// implementation of Compressor for LZMA
// implementation of Compressor for Zstd
