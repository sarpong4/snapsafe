use std::io::{self, Read, Write};

use flate2::{read::{GzDecoder, ZlibDecoder}, write::{GzEncoder, ZlibEncoder}, Compression};

// algorithm levels
// gzip/zlib -> use default (6)
// brotli -> use balanced (6)
// zstd -> use balanced (3)

#[derive(Clone)]
pub enum CompressionType {
    None,
    Gzip,
    Zlib,
    // Future: Brotli, LZMA, Zstd, etc.
}

pub trait Compressor {
    fn compress(&self, data: &[u8]) -> io::Result<(Vec<u8>, CompressionDetail)>;
    fn decompress(&self, data: &[u8]) -> io::Result<Vec<u8>>;
}

pub struct CompressionDetail {
    compression_algorithm: CompressionType,
    level: usize,
}

impl CompressionDetail {
    pub fn new(algo: CompressionType) -> Self {
        let level = match algo {
            CompressionType::Gzip | CompressionType::None | CompressionType::Zlib => 6
        };

        Self { compression_algorithm: algo, level }
    }
}

// pub struct ZstdCompressor;
impl Compressor for CompressionType {
    fn compress(&self, data: &[u8]) -> io::Result<(Vec<u8>, CompressionDetail)> {
        match self {
            CompressionType::Gzip | CompressionType::None => {
                let mut encoder = 
                    GzEncoder::new(Vec::new(), Compression::new(6));
                encoder.write_all(data)?;
                let compressed = encoder.finish()?;
                return Ok((compressed, CompressionDetail::new(self.clone())));
            },
            CompressionType::Zlib => {
                let mut encoder = 
                    ZlibEncoder::new(Vec::<u8>::new(), Compression::default());
                encoder.write_all(data)?;
                let compressed = encoder.finish()?;
                return Ok((compressed, CompressionDetail::new(self.clone())))
            },
        }
    }
    fn decompress(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        match self {
            CompressionType::Gzip | CompressionType::None => {
                let mut decoder = GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                return Ok(decompressed)
            },
            CompressionType::Zlib => {
                let mut decoder = ZlibDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                return Ok(decompressed)
            }
        }
    }
}
