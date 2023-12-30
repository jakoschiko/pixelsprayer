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
        device: Option<String>,
        tcp_nodelay: bool,
    ) -> Result<Self> {
        let socket = TcpSocket::new_v4()?;
        if let Some(bind) = bind {
            socket.bind(bind)?;
        }

        socket.bind_device(device.as_ref().map(|s| s.as_bytes()))?;
        println!("DEBUG: device={:?}", socket.device()?);

        let stream = socket.connect(connect).await?;

        println!(
            "DEBUG: local_addr={:?}, peer_addr={:?}",
            stream.local_addr()?,
            stream.peer_addr()?,
        );

        stream.set_nodelay(tcp_nodelay)?;

        let send_buffer = BytesMut::new();
        Ok(Self {
            stream,
            send_buffer,
        })
    }

    pub fn enqueue_pixel(&mut self, position: Position, color: Color) -> Result<()> {
        let Position { x, y } = position;
        match color {
            Color::None => (),
            Color::Grayscale(c) => writeln!(&mut self.send_buffer, "PX {x} {y} {c:02x}")?,
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
