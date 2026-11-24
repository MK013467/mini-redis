mod resp;
#[cfg(test)]
mod test;

use crate::resp::{ParseError, RespValue, decode};
use std::{
    io::{BufReader, Read, Result, Write},
    net::{TcpListener, TcpStream},
};

const MAX_BYTE: usize = 1024;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    for stream in listener.incoming() {
        // Todo: error handling
        if let Err(error) = stream.and_then(handle_connection) {
            eprintln!("{error}");
        }
    }
    Ok(())
}

fn handle_connection(stream: TcpStream) -> Result<()> {
    let mut stream = BufReader::new(stream);
    let mut buffer: Vec<u8> = Vec::new();
    let mut chunk: [u8; MAX_BYTE] = [0u8; MAX_BYTE];

    loop {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            if !buffer.is_empty() {
                return Err(std::io::ErrorKind::UnexpectedEof.into());
            }
            return Ok(());
        }
        buffer.extend_from_slice(&chunk[..n]);
        while !buffer.is_empty() {
            match decode(&buffer) {
                Ok((value, used)) => {
                    buffer.drain(..used);
                    let response = handle_command(&value);
                    stream.get_mut().write_all(&response)?;
                }

                Err(ParseError::Incomplete) => {
                    break;
                }
                Err(_) => return Err(std::io::ErrorKind::InvalidData.into()),
            }
        }
    }
}

// all redis commands are arrays
fn handle_command(value: &RespValue) -> Vec<u8> {
    let RespValue::Array(items) = value else {
        return b"Expected a array\r\n".to_vec();
    };

    match items.as_slice() {
        [RespValue::BulkString(Some(command))] if command.eq_ignore_ascii_case(b"PING") => {
            b"+PONG\r\n".to_vec()
        }
        [
            RespValue::BulkString(Some(command)),
            RespValue::BulkString(Some(message)),
        ] if command.eq_ignore_ascii_case(b"ECHO") => {
            let mut response = format!("${}\r\n", message.len()).into_bytes();
            response.extend_from_slice(message);
            response.extend_from_slice(b"\r\n");
            response
        }
        _ => b"Todo later".to_vec(),
    }
}
