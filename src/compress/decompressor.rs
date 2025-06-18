use std::io::{self, Read};

use flate2::read::{GzDecoder, ZlibDecoder};

/**
 *  Decompressor trait implemented by the various decompression algorithms
 * For any algorithm we implement, you need to call the decompress function on them.
 * For GzDecoder, we only need to call `GzDecoder::decompress(data)` 
 * then the implementation is abstracted from the user. 
 * Each call will return a `Result<Vec<u8>, Error>` where `Vec<u8>` is the buffer the data read is placed into.
*/
pub trait Decompressor<R> {
    fn decompress(data: R) -> io::Result<Vec<u8>>;
}

impl<R: Read> Decompressor<R> for ZlibDecoder<R> {
    fn decompress(data: R) -> io::Result<Vec<u8>> {
        let mut decoder = Self::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}

impl<R: Read> Decompressor<R> for GzDecoder<R> {
    fn decompress(data: R) -> io::Result<Vec<u8>> {
        let mut decoder = Self::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}
