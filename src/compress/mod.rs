use std::io;

use flate2::{read::{GzDecoder, ZlibDecoder}, write::{GzEncoder, ZlibEncoder}};

use crate::compress::{compressor::{CompressionType, Compressor}, decompressor::Decompressor};

pub mod compressor;
pub mod decompressor;

// algorithm levels
// gzip/zlib -> use default (6)
// brotli -> use balanced (6)
// zstd -> use balanced (3)

pub trait CompressDecompress {
    fn compress(&self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn decompress(&self, data: &[u8]) -> io::Result<Vec<u8>>;
}

impl CompressDecompress for CompressionType {
    fn compress(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        match self {
            CompressionType::Gzip | CompressionType::None => GzEncoder::compress(data),
            CompressionType::Zlib => ZlibEncoder::compress(data),
        }
    }
    fn decompress(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        match self {
            CompressionType::Gzip | CompressionType::None => GzDecoder::decompress(data),
            CompressionType::Zlib => ZlibDecoder::decompress(data),
        }
    }
}
