use std::io;

pub mod compressor;
pub mod decompressor;

pub use compressor::{CompressionType, CompressionEngine};

use crate::compress::decompressor::DecompressionEngine;

// algorithm levels
// gzip/zlib -> use default (6)
// brotli -> use balanced (6)
// zstd -> use balanced (3)
// lzma -> use balanced (7)

pub fn build_compress_engine() -> CompressionEngine {
    // parse config according to overriding definition:
    // check if it is part of the command line arguments first
    // if not check global config file if none
    // check local config
    CompressionEngine::new(CompressionType::Brotli, 6)
}

pub fn build_decompression_engine() -> DecompressionEngine {

    DecompressionEngine::new(CompressionType::Brotli)
}

pub fn compress(data: &[u8], algorithm: CompressionType, level: u32) -> io::Result<Vec<u8>> {
    let engine = CompressionEngine::new(algorithm, level);
    engine.compress(data)
}

pub fn decompress(data: &[u8], algorithm: CompressionType) -> io::Result<Vec<u8>> {
    let engine = DecompressionEngine::new(algorithm);
    engine.decompress(data)
}
