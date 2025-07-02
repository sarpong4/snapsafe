use std::str::FromStr;

pub mod compressor;
pub mod decompressor;

pub use compressor::{CompressionType, CompressionEngine};

use crate::{compress::decompressor::DecompressionEngine, utils::error::SnapError};

// algorithm levels
// gzip/zlib -> use default (6)
// brotli -> use balanced (7)
// zstd -> use balanced (3)
// lzma -> use balanced (7)

fn get_algorithm_levels(algo: &CompressionType) -> u32 {
    match algo {
        CompressionType::None | CompressionType::Gzip | CompressionType::Zlib => 6,
        CompressionType::Brotli => 7,
        CompressionType::Zstd => 3,
        CompressionType::Lzma => 7,
    }
}

pub fn build_compress_engine(comp: String) -> Result<CompressionEngine, SnapError> {
    // parse config according to overriding definition:
    // check if it is part of the command line arguments first
    // if not check global config file if none
    // check local config

    let algo = get_compression_type(comp);

    if let Err(err) = algo {
        return Err(err);
    }

    let algo = algo.ok().unwrap();
    let level = get_algorithm_levels(&algo);

    Ok(CompressionEngine::new(algo, level))
}

pub fn get_compression_type(comp: String) -> Result<CompressionType, SnapError> {
    let compression_type = CompressionType::from_str(comp.as_str());

    if let Err(_) = compression_type {
        let message = "Defined algorithm choice unavailble";
        let err = SnapError::Config(message.into());
        return Err(err);
    }

    Ok(compression_type.unwrap())
}

pub fn build_decompression_engine(comp: String) -> Result<DecompressionEngine, SnapError> {

    let algo = get_compression_type(comp);

    if let Err(err) = algo {
        return Err(err);
    }

    let algo = algo.ok().unwrap();

    Ok(DecompressionEngine::new(algo))
}
