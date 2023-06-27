#![no_std]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]
#![doc(issue_tracker_base_url = "https://github.com/Finomnis/ws2812-flexio/issues")]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Blocking driver.
pub mod blocking;

/// FlexIO driver
mod pixel;
mod prepared_pixels;

pub use pixel::Pixel;
pub use prepared_pixels::PreparedPixels;
