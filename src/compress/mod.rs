pub mod compress_engine;

use std::str::FromStr;

pub use compress_engine::{CompressionType, CompressionEngine};

use crate::{compress::{compress_engine::{BrotliEngine, GzipEngine, LzmaEngine, NullEngine, ZlibEngine, ZstdEngine}}, utils::error::SnapError};

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

pub fn build_engine(comp: String) -> Result<Box<dyn CompressionEngine>, SnapError> {
    // parse config according to overriding definition:
    // check if it is part of the command line arguments first
    // if not check global config file if none
    // check local config

    let algo = get_compression_type(comp)?;
    let level = get_algorithm_levels(&algo);

    let engine: Box<dyn CompressionEngine> = match algo {
        CompressionType::None => Box::new(NullEngine),
        CompressionType::Gzip => Box::new(GzipEngine { level }),
        CompressionType::Zlib => Box::new(ZlibEngine { level }),
        CompressionType::Brotli => Box::new(BrotliEngine { level }),
        CompressionType::Zstd => Box::new(ZstdEngine { level: level as i32 }),
        CompressionType::Lzma => Box::new(LzmaEngine { level }),
    };

    Ok(engine)
}

pub fn get_compression_type(comp: String) -> Result<CompressionType, SnapError> {
    let compression_type = CompressionType::from_str(comp.as_str())?;

    Ok(compression_type)
}
