use std::{fmt::Write, net::SocketAddr};

use anyhow::{Error, Result};
use bytes::BytesMut;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpSocket, TcpStream},
};

use crate::{color::Color, position::Position};

pub struct Client {
    stream: TcpStream,
    send_buffer: BytesMut,
}

impl Client {
    pub async fn connect(
        connect: SocketAddr,
        bind: Option<SocketAddr>,
        nodelay: bool,
    ) -> Result<Self> {
        let socket = TcpSocket::new_v4()?;
        if let Some(bind) = bind {
            socket.bind(bind)?;
        };
        let stream = socket.connect(connect).await?;

        stream.set_nodelay(nodelay)?;

        let send_buffer = BytesMut::new();
        Ok(Self {
            stream,
            send_buffer,
        })
    }

    pub fn enqueue_pixel(
        &mut self,
        position: Position,
        color: Color,
        optimize_grayscale_rgb: bool,
    ) -> Result<()> {
        let Position { x, y } = position;
        match color.normalize() {
            Color::None => (),
            Color::Grayscale(c) => {
                if optimize_grayscale_rgb {
                    writeln!(&mut self.send_buffer, "PX {x} {y} {c:02x}")?
                } else {
                    writeln!(&mut self.send_buffer, "PX {x} {y} {c:02x}{c:02x}{c:02x}")?
                }
            }
            Color::Rgb(r, g, b) => {
                writeln!(&mut self.send_buffer, "PX {x} {y} {r:02x}{g:02x}{b:02x}")?
            }
            Color::Rgba(r, g, b, a) => writeln!(
                &mut self.send_buffer,
                "PX {x} {y} {r:02x}{g:02x}{b:02x}{a:02x}"
            )?,
        };

        Ok(())
    }

    pub async fn progress(&mut self, min_bytes_for_sending: u32) -> Result<()> {
        while !self.send_buffer.is_empty()
            && self.send_buffer.len() >= min_bytes_for_sending as usize
        {
            let written_bytes = self.stream.write_buf(&mut self.send_buffer).await?;

            if written_bytes == 0 {
                return Err(Error::msg("Connection closed"));
            }
        }
        Ok(())
    }
}
