use std::{io::{Read, Write}, str::FromStr};

use brotli2::{read::BrotliDecoder, write::BrotliEncoder};
use flate2::{read::{GzDecoder, ZlibDecoder}, write::{GzEncoder, ZlibEncoder}, Compression};
use xz2::write::XzEncoder as LzmaEncoder;
use zstd::stream::Encoder as ZstdEncoder;

use xz2::read::XzDecoder as LzmaDecoder;
use zstd::stream::Decoder as ZstdDecoder;

use crate::utils::error::SnapError;

#[derive(Clone, Debug, PartialEq)]
pub enum CompressionType {
    None,
    Gzip,
    Zlib,
    Brotli,
    Zstd,
    Lzma,
}

pub trait CompressionEngine {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError>;
}

impl FromStr for CompressionType {
    type Err = SnapError;

    fn from_str(algorithm: &str) -> Result<Self, Self::Err> {
        match algorithm.to_lowercase().as_str() {
            "gzip" => Ok(CompressionType::Gzip),
            "zlib" => Ok(CompressionType::Zlib),
            "brotli" => Ok(CompressionType::Brotli),
            "zstd" => Ok(CompressionType::Zstd),
            "lzma" => Ok(CompressionType::Lzma),
            "none" => Ok(CompressionType::None),
            _ => Err(SnapError::InvalidCompressor(algorithm.into()))
        }
    }
}

pub struct NullEngine;

impl CompressionEngine for NullEngine {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        Ok(data.to_vec())
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        Ok(data.to_vec())
    }
}

pub struct GzipEngine { pub level: u32 }

impl CompressionEngine for GzipEngine {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.level));
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}
pub struct BrotliEngine { pub level: u32 }

impl CompressionEngine for BrotliEngine{
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        let mut encoder = BrotliEncoder::new(Vec::new(), self.level);
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        let mut decoder = BrotliDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}

pub struct ZlibEngine { pub level: u32 }

impl CompressionEngine for ZlibEngine {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(self.level));
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}

pub struct ZstdEngine { pub level: i32 }

impl CompressionEngine for ZstdEngine {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        let mut encoder = ZstdEncoder::new(Vec::new(), self.level)?;
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        let mut decoder = ZstdDecoder::new(data)?;
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}

pub struct LzmaEngine { pub level: u32 }

impl CompressionEngine for LzmaEngine {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        let mut encoder = LzmaEncoder::new(Vec::new(), self.level);
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, SnapError> {
        let mut decoder = LzmaDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}
