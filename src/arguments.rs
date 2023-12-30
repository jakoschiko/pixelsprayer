use std::net::SocketAddr;

use argh::FromArgs;

#[derive(FromArgs)]
/// Pixelsprayer.
pub struct Arguments {
    /// connect
    #[argh(positional)]
    pub connect: SocketAddr,

    /// image (path)
    #[argh(positional)]
    pub image_path: String,

    /// worker count
    #[argh(positional, default = "3")]
    pub worker_count: u64,

    /// x offset
    #[argh(option, default = "0")]
    pub x: u32,

    /// y offset
    #[argh(option, default = "0")]
    pub y: u32,

    /// minimum bytes that will be written to TCP
    #[argh(option, default = "1400")]
    pub min_bytes_for_sending: u32,

    /// optimize grayscale RGB
    #[argh(switch)]
    pub optimize_grayscale_rgb: bool,

    /// bind
    #[argh(option)]
    pub bind: Option<SocketAddr>,

    /// nodelay TCP
    #[argh(switch)]
    pub nodelay: bool,
}
