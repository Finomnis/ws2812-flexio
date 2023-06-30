#![no_std]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]
#![doc(issue_tracker_base_url = "https://github.com/Finomnis/ws2812-flexio/issues")]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Blocking driver.
mod flexio;
mod pins;
mod pixel;
mod pixelstream;

/// Possible errors that could happen.
pub mod errors;

pub use flexio::{PreprocessedPixels, WS2812Driver, WriteDmaResult};
pub use pins::Pins;
pub use pixel::Pixel;
pub use pixelstream::IntoPixelStream;
