use std::{
    io::{BufRead, BufReader, Read, Result, Write},
    net::{TcpListener, TcpStream},
};

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    for stream in listener.incoming() {
        if let Err(error) = stream.and_then(handle_connection) {
            eprintln!("{error}");
        }
    }
    Ok(())
}

fn handle_connection(stream: TcpStream) -> Result<()> {
    let mut stream = BufReader::new(stream);
    let mut line = Vec::new();

    loop {
        line.clear();
        if stream.read_until(b'\n', &mut line)? == 0 {
            return Ok(());
        }

        // Temporary PING handler until the RESP decoder is ready.
        let is_ping = if line == b"*1\r\n" {
            let mut bulk = [0; b"$4\r\nPING\r\n".len()];
            stream.read_exact(&mut bulk)?;
            bulk.eq_ignore_ascii_case(b"$4\r\nPING\r\n")
        } else {
            line.eq_ignore_ascii_case(b"PING\r\n") || line.eq_ignore_ascii_case(b"PING\n")
        };

        if !is_ping {
            stream.get_mut().write_all(b"-ERR expected PING\r\n")?;
            return Ok(());
        }
        stream.get_mut().write_all(b"+PONG\r\n")?;
    }
}
