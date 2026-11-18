#[derive(Debug)]
pub enum ParseError {
    Incomplete,
    InvalidPrefix,
    InvalidLength,
    InvalidFormat,
}

pub enum RespValue {
    SimpleString(String),
    BulkString(Option<Vec<u8>>),
    Array(Vec<RespValue>),
}


fn find_crlf( input: &[u8]) -> Option<usize>{
    input.windows(2).position(|item|  item == b"\r\n") 
}


pub fn decode(input: &[u8]) -> Result<(RespValue, usize), ParseError> {
    if input.is_empty() {
        return Err(ParseError::InvalidFormat);
    }

    match input[0] {
        b'+' => {
            let Some(num) = find_crlf(input) else{
                return Err(ParseError::InvalidFormat);
            };
            
            let clone = input.clone();
            let s = std::str::from_utf8(&clone[..num]).unwrap();
                return Ok((
                    RespValue::SimpleString(s.to_string())
                    , num - 1 
                ));
            
        }
        b'$' => {
            // Bulk String 파싱
            return Ok()
        }
        b'*' => {
            // Array 파싱
            return Ok(None)
        }
        _ =>{ 
            println!("error");
            Err(ParseError::InvalidPrefix)
        }}
}
