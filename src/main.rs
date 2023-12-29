mod client;
mod color;
mod image;
mod position;

use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::Path,
    sync::Arc,
};

use anyhow::Result;
use argh::FromArgs;
use fastrand::Rng;
use image::Image;
use position::Position;
use tokio::task::JoinSet;

use crate::client::Client;

#[derive(FromArgs)]
/// Pixelsprayer.
struct Arguments {
    /// host
    #[argh(positional)]
    host: String,

    /// port
    #[argh(positional)]
    port: u16,

    /// image (path)
    #[argh(positional)]
    image_path: String,

    /// worker count
    #[argh(positional, default = "32")]
    worker_count: u64,

    /// x offset
    #[argh(option, default = "0")]
    x: u32,

    /// y offset
    #[argh(option, default = "0")]
    y: u32,

    /// optimize grayscale RGB
    #[argh(switch)]
    optimize_grayscale_rgb: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Arguments = argh::from_env();

    let host: SocketAddr = format!("{}:{}", args.host, args.port)
        .as_str()
        .to_socket_addrs()?
        .collect::<Vec<_>>()[0];
    let image_path = Path::new(&args.image_path);
    let offset = Position {
        x: args.x,
        y: args.y,
    };
    let image = Arc::new(Image::open(image_path)?);

    let mut set = JoinSet::new();
    for id in 0..args.worker_count {
        set.spawn(run_worker(
            id,
            host,
            image.clone(),
            offset,
            args.optimize_grayscale_rgb,
        ));
    }
    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(())) => (),
            Ok(Err(error)) => {
                println!("Task failed: {error}");
            }
            Err(error) => {
                println!("Failed to join with service task: {error}");
            }
        }
    }
    Ok(())
}

async fn run_worker(
    id: u64,
    host: SocketAddr,
    image: Arc<Image>,
    offset: Position,
    optimize_grayscale_rgb: bool,
) -> Result<()> {
    // TODO: Reconnect on error?
    // TODO: Improve logging
    let mut rng = Rng::with_seed(id);
    println!("{id} Connecting");
    let mut client = Client::connect(host).await?;
    println!("{id} Start sending pixels");
    loop {
        let position = offset.add(image.get_random_position(&mut rng));
        if let Some(color) = image.get_color(position) {
            client
                .set_pixel(position, color, optimize_grayscale_rgb)
                .await?;
        }
    }
}
