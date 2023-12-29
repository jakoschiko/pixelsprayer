use std::{io::Write, net::SocketAddr};

use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{color::Color, position::Position};

pub struct Client {
    stream: TcpStream,
    send_buffer: Vec<u8>,
}

impl Client {
    pub async fn connect(host: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(host).await?;
        let send_buffer = Vec::new();
        Ok(Self {
            stream,
            send_buffer,
        })
    }

    pub async fn set_pixel(
        &mut self,
        position: Position,
        color: Color,
        optimize_grayscale_rgb: bool,
    ) -> Result<()> {
        self.send_buffer.clear();

        let Position { x, y } = position;
        match color.normalize() {
            Color::Grayscale(c) => {
                if optimize_grayscale_rgb {
                    writeln!(&mut self.send_buffer, "PX {x} {y} {c:02x}")?
                } else {
                    writeln!(&mut self.send_buffer, "PX {x} {y} {c:02x}{c:02x}{c:02x}")?
                }
            }
            Color::Rgb(r, g, b) => {
                // TODO: support format without alpha
                writeln!(&mut self.send_buffer, "PX {x} {y} {r:02x}{g:02x}{b:02x}ff")?
            }
            Color::Rgba(r, g, b, a) => writeln!(
                &mut self.send_buffer,
                "PX {x} {y} {r:02x}{g:02x}{b:02x}{a:02x}"
            )?,
        };

        self.stream.write_all(&self.send_buffer).await?;

        Ok(())
    }
}
