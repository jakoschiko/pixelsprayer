mod arguments;
mod client;
mod color;
mod image;
mod position;

use std::{net::SocketAddr, path::Path, sync::Arc};

use anyhow::Result;
use arguments::Arguments;
use fastrand::Rng;
use image::Image;
use position::Position;
use tokio::task::JoinSet;

use crate::client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Arguments = argh::from_env();

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
            args.connect,
            image.clone(),
            offset,
            args.min_bytes_for_sending,
            args.optimize_grayscale_rgb,
            args.bind,
            args.nodelay,
        ));
    }
    while let Some(result) = set.join_next().await {
        match result {
            Ok(()) => (),
            Err(error) => {
                println!("Failed to join with worker task: {error}");
            }
        }
    }
    Ok(())
}

async fn run_worker(
    id: u64,
    connect: SocketAddr,
    image: Arc<Image>,
    offset: Position,
    min_bytes_for_sending: u32,
    optimize_grayscale_rgb: bool,
    bind: Option<SocketAddr>,
    nodelay: bool,
) {
    // TODO: Improve logging
    loop {
        let result = try_run_worker(
            id,
            connect,
            &image,
            offset,
            min_bytes_for_sending,
            optimize_grayscale_rgb,
            bind,
            nodelay,
        )
        .await;

        match result {
            Ok(()) => (),
            Err(error) => {
                println!("Worker {id}: {error}");
            }
        }
    }
}

async fn try_run_worker(
    id: u64,
    connect: SocketAddr,
    image: &Image,
    offset: Position,
    min_bytes_for_sending: u32,
    optimize_grayscale_rgb: bool,
    bind: Option<SocketAddr>,
    nodelay: bool,
) -> Result<()> {
    let mut rng = Rng::with_seed(id);
    println!("Worker {id}: Start connecting");
    let mut client = Client::connect(connect, bind, nodelay).await?;
    println!("Worker {id}: Start sending pixels");
    loop {
        let position = image.get_random_position(&mut rng);
        if let Some(color) = image.get_color(position) {
            client.enqueue_pixel(position.add(offset), color, optimize_grayscale_rgb)?;
        }

        client.progress(min_bytes_for_sending).await?;
    }
}
