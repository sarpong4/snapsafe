use std::io::{self, Read};

use brotli2::read::BrotliDecoder;
use flate2::read::{GzDecoder, ZlibDecoder};
use xz2::read::XzDecoder as LzmaDecoder;
use zstd::stream::Decoder as ZstdDecoder;

use crate::compress::CompressionType;

pub struct DecompressionEngine {
    algorithm: CompressionType,
}

impl DecompressionEngine {
    pub fn new(algo: CompressionType) -> Self {
        Self {
            algorithm: algo,
        }
    }

    pub fn decompress(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        match self.algorithm {
            CompressionType::None => Ok(data.to_vec()),
            
            CompressionType::Gzip => {
                let mut decoder = GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            },

            CompressionType::Zlib => {
                let mut decoder = ZlibDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            },

            CompressionType::Brotli => {
                let mut decoder = BrotliDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            },

            CompressionType::Zstd => {
                let mut decoder = ZstdDecoder::new(data)?;
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            },

            CompressionType::Lzma => {
                let mut decoder = LzmaDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
        }
    }
}
