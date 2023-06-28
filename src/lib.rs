#![no_std]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]
#![doc(issue_tracker_base_url = "https://github.com/Finomnis/ws2812-flexio/issues")]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Blocking driver.
pub mod blocking;

mod flexio_configurator;
mod pins;
mod pixel;
mod pixelstream;

/// Possible errors that could happen.
pub mod errors;

pub use pins::Pins;
pub use pixel::{Pixel, PixelBytes};
use pixelstream::PixelStream;
