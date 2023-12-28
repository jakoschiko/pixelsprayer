mod client;
mod color;
mod image;
mod position;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use fastrand::Rng;
use image::Image;
use position::Position;
use tokio::task::JoinSet;

use crate::client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Read from args
    let host = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1337);
    let image_path = Path::new("image.png");
    let offset = Position { x: 0, y: 0 };
    let worker_count = 32;

    let image = Arc::new(Image::open(image_path)?);

    let mut set = JoinSet::new();
    for id in 0..worker_count {
        set.spawn(run_worker(id, host, image.clone(), offset));
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

async fn run_worker(id: u64, host: SocketAddr, image: Arc<Image>, offset: Position) -> Result<()> {
    // TODO: Reconnect on error?
    // TODO: Improve logging
    let mut rng = Rng::with_seed(id);
    println!("{id} Connecting");
    let mut client = Client::connect(host).await?;
    println!("{id} Start sending pixels");
    loop {
        let position = offset.add(image.get_random_position(&mut rng));
        if let Some(color) = image.get_color(position) {
            client.set_pixel(position, color).await?;
        }
    }
}
