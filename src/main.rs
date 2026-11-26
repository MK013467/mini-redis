mod resp;
mod storage;
#[cfg(test)]
mod test;

use crate::resp::{ParseError, RespValue, decode};
use crate::storage::Storage;
use std::{
    io::{BufReader, Read, Result, Write},
    net::{TcpListener, TcpStream},
};

const MAX_BYTE: usize = 1024;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let mut storage = Storage::default();
    for stream in listener.incoming() {
        // Todo: error handling
        if let Err(error) = stream.and_then(|stream| handle_connection(stream, &mut storage)) {
            eprintln!("{error}");
        }
    }
    Ok(())
}

fn handle_connection(stream: TcpStream, storage: &mut Storage) -> Result<()> {
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
                    let response = handle_command(&value, storage);
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

fn encode_bulk_string(value: &[u8]) -> Vec<u8> {
    let mut response = format!("${}\r\n", value.len()).into_bytes();
    response.extend_from_slice(value);
    response.extend_from_slice(b"\r\n");
    response
}

fn handle_command(value: &RespValue, storage: &mut Storage) -> Vec<u8> {
    use RespValue::BulkString;

    let RespValue::Array(items) = value else {
        return b"-ERR expected a command array\r\n".to_vec();
    };

    match items.as_slice() {
        [BulkString(Some(command))] if command.eq_ignore_ascii_case(b"PING") => {
            b"+PONG\r\n".to_vec()
        }
        [BulkString(Some(command)), BulkString(Some(arg))] => {
            if command.eq_ignore_ascii_case(b"ECHO") {
                encode_bulk_string(arg)
            } else if command.eq_ignore_ascii_case(b"GET") {
                match storage.get(arg) {
                    Some(value) => encode_bulk_string(value),
                    None => b"$-1\r\n".to_vec(),
                }
            } else if command.eq_ignore_ascii_case(b"DEL") {
                if storage.del(arg) {
                    b":1\r\n".to_vec()
                } else {
                    b":0\r\n".to_vec()
                }
            } else {
                b"-ERR unsupported command\r\n".to_vec()
            }
        }
        [
            BulkString(Some(command)),
            BulkString(Some(key)),
            BulkString(Some(value)),
        ] if command.eq_ignore_ascii_case(b"SET") => {
            storage.set(key.to_vec(), value.to_vec());
            b"+OK\r\n".to_vec()
        }
        _ => b"-ERR unsupported command or wrong number of arguments\r\n".to_vec(),
    }
}
