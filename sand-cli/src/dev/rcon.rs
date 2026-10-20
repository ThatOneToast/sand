//! Minimal bounded Minecraft RCON transport. Credentials never implement Debug.
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::time::Duration;

use anyhow::{Context, Result, ensure};

pub(super) struct Rcon {
    pub port: u16,
    secret: String,
}

impl Rcon {
    pub fn new() -> Result<Self> {
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let port = listener.local_addr()?.port();
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|_| anyhow::anyhow!("OS entropy unavailable"))?;
        let secret = bytes.iter().map(|b| format!("{b:02x}")).collect();
        Ok(Self { port, secret })
    }

    pub fn properties(&self) -> String {
        format!(
            "server-ip=127.0.0.1\nenable-rcon=true\nrcon.port={}\nrcon.password={}\nbroadcast-rcon-to-ops=false\n",
            self.port, self.secret
        )
    }

    pub fn reload(&self) -> Result<()> {
        let mut stream = TcpStream::connect_timeout(
            &SocketAddrV4::new(Ipv4Addr::LOCALHOST, self.port).into(),
            Duration::from_secs(2),
        )
        .context("cannot connect to local RCON")?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        write_packet(&mut stream, 1, 3, &self.secret)?;
        let (id, kind) = read_packet(&mut stream)?;
        ensure!(id == 1 && kind == 2, "local RCON authentication rejected");
        write_packet(&mut stream, 2, 2, "reload")?;
        let (id, kind) = read_packet(&mut stream)?;
        ensure!(id == 2 && kind == 0, "unexpected RCON command response");
        // Vanilla's response acknowledges dispatch, not asynchronous resource
        // validation. Never label this as successful datapack loading.
        Ok(())
    }
}

fn write_packet(out: &mut impl Write, id: i32, kind: i32, body: &str) -> Result<()> {
    ensure!(body.len() <= 4096, "RCON request too large");
    out.write_all(&(body.len() as i32 + 10).to_le_bytes())?;
    out.write_all(&id.to_le_bytes())?;
    out.write_all(&kind.to_le_bytes())?;
    out.write_all(body.as_bytes())?;
    out.write_all(&[0, 0])?;
    out.flush()?;
    Ok(())
}

fn read_packet(input: &mut impl Read) -> Result<(i32, i32)> {
    let mut length = [0; 4];
    input.read_exact(&mut length)?;
    let length = i32::from_le_bytes(length);
    ensure!((10..=4110).contains(&length), "invalid RCON packet length");
    let mut packet = vec![0; length as usize];
    input.read_exact(&mut packet)?;
    ensure!(packet.ends_with(&[0, 0]), "invalid RCON terminator");
    Ok((
        i32::from_le_bytes(packet[..4].try_into()?),
        i32::from_le_bytes(packet[4..8].try_into()?),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_truncated_oversized_and_unterminated_packets() {
        for length in [-1i32, 0, 9, 4111, i32::MAX] {
            assert!(read_packet(&mut length.to_le_bytes().as_slice()).is_err());
        }
        let mut bytes = Vec::new();
        write_packet(&mut bytes, 3, 0, "ok").unwrap();
        assert_eq!(read_packet(&mut bytes.as_slice()).unwrap(), (3, 0));
        bytes.pop();
        assert!(read_packet(&mut bytes.as_slice()).is_err());
        bytes.push(1);
        assert!(read_packet(&mut bytes.as_slice()).is_err());
    }
    #[test]
    fn authentication_failure_never_sends_reload() {
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let worker = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            assert_eq!(read_packet(&mut socket).unwrap(), (1, 3));
            write_packet(&mut socket, -1, 2, "").unwrap();
            let mut byte = [0];
            assert_eq!(socket.read(&mut byte).unwrap(), 0);
        });
        let rcon = Rcon {
            port,
            secret: "test-only".into(),
        };
        let error = rcon.reload().unwrap_err().to_string();
        assert!(error.contains("authentication rejected"));
        assert!(!error.contains(&rcon.secret));
        worker.join().unwrap();
    }

    #[test]
    fn credentials_are_fresh_and_server_binding_is_loopback() {
        let a = Rcon::new().unwrap();
        let b = Rcon::new().unwrap();
        assert!(a.secret != b.secret);
        assert_eq!(a.secret.len(), 64);
        assert!(a.properties().contains("server-ip=127.0.0.1\n"));
    }
}
