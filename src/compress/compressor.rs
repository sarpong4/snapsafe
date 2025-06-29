use std::{io::{self, Write}, str::FromStr};

use brotli2::{write::BrotliEncoder};
use flate2::{write::{GzEncoder, ZlibEncoder}, Compression};
use xz2::write::XzEncoder as LzmaEncoder;
use zstd::stream::Encoder as ZstdEncoder;

#[derive(Clone, Debug, PartialEq)]
pub enum CompressionType {
    None,
    Gzip,
    Zlib,
    Brotli,
    Zstd,
    Lzma,
}

pub struct CompressionEngine {
    algorithm: CompressionType,
    level: u32
}

impl FromStr for CompressionType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "none" => Ok(CompressionType::None),
            "gzip" => Ok(CompressionType::Gzip),
            "zlib" => Ok(CompressionType::Zlib),
            "brotli" => Ok(CompressionType::Brotli),
            "zstd" => Ok(CompressionType::Zstd),
            "lzma" => Ok(CompressionType::Lzma),
            _ => Err(()),
        }
    }
}

impl CompressionEngine {
    pub fn new(algo: CompressionType, level: u32) -> Self {
        Self {
            algorithm: algo,
            level
        }
    }

    pub fn compress(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        match self.algorithm {
            CompressionType::None => Ok(data.to_vec()),
            
            CompressionType::Gzip => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.level));
                encoder.write_all(&data)?;
                encoder.finish()
            },

            CompressionType::Zlib => {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(self.level));
                encoder.write_all(&data)?;
                encoder.finish()
            },

            CompressionType::Brotli => {
                let mut encoder = BrotliEncoder::new(Vec::new(), self.level);
                encoder.write_all(data)?;
                encoder.finish()
            },

            CompressionType::Zstd => {
                let mut encoder = ZstdEncoder::new(Vec::new(), self.level as i32)?;
                encoder.write_all(data)?;
                encoder.finish()
            },

            CompressionType::Lzma => {
                let mut encoder = LzmaEncoder::new(Vec::new(), self.level);
                encoder.write_all(&data)?;
                encoder.finish()
            }
        }
    }
}
