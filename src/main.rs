mod arguments;
mod client;
mod color;
mod image;
mod position;

use std::{path::Path, sync::Arc};

use anyhow::Result;
use arguments::Arguments;
use fastrand::Rng;
use image::Image;
use position::Position;
use tokio::task::JoinSet;

use crate::client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arc::new(argh::from_env::<Arguments>());

    let image_path = Path::new(&args.image_path);
    let image = Arc::new(Image::open(image_path)?);

    let mut set = JoinSet::new();
    for id in 0..args.worker_count {
        set.spawn(run_worker(id, args.clone(), image.clone()));
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

async fn run_worker(id: u64, args: Arc<Arguments>, image: Arc<Image>) {
    // TODO: Improve logging
    loop {
        let result = try_run_worker(id, &args, &image).await;

        match result {
            Ok(()) => (),
            Err(error) => {
                println!("Worker {id}: {error}");
            }
        }
    }
}

async fn try_run_worker(id: u64, args: &Arguments, image: &Image) -> Result<()> {
    let mut rng = Rng::with_seed(id);
    println!("Worker {id}: Start connecting");
    let mut client = Client::connect(
        args.connect,
        args.bind,
        args.device.clone(),
        args.tcp_nodelay,
    )
    .await?;
    println!("Worker {id}: Start sending pixels");
    loop {
        let position = image.get_random_position(&mut rng);
        if let Some(color) = image.get_color(position) {
            let offset = Position {
                x: args.x,
                y: args.y,
            };
            let color = color.normalize(args.disable_grayscale_support);
            client.enqueue_pixel(position.add(offset), color)?;
        }

        client.progress(args.min_bytes_for_sending).await?;
    }
}
