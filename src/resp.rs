#[derive(Debug)]
pub enum ParseError {
    Incomplete,
    InvalidPrefix,
    InvalidLength,
    InvalidFormat,
}

pub enum RespValue {
    Integer(i64),
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
        // Integer
        b':' => {
          let end = find_crlf(input).ok_or(ParseError::Incomplete)?;
            let text = std::str::from_utf8(&input[1..end])
                .map_err(|_| ParseError::InvalidFormat)?;
            let result = text
                .parse::<i64>()
                .map_err(|_| ParseError::InvalidFormat)?;
                
            Ok((RespValue::Integer(result), end + 2))
        }

        // Simple String
        b'+' => {
            let Some(num) = find_crlf(input) else{
                return Err(ParseError::InvalidFormat);
            };
            
            let clone = input.clone();
            let s = std::str::from_utf8(&clone[1..num]).unwrap();
                return Ok((
                    RespValue::SimpleString(s.to_string())
                    , num - 1 
                ));
            
        }
        // Bulk String
        b'$' => {
            let header_end = find_crlf(input).ok_or(ParseError::Incomplete)?;
            let len_b = &input[1..header_end];
            let body_start = header_end + 2;

            if len_b == b"-1" {
                return Ok((RespValue::BulkString(None), body_start));
            }

            if !len_b.iter().all(u8::is_ascii_digit) {
                return Err(ParseError::InvalidLength);
            }

            let len = std::str::from_utf8(len_b)
                .map_err(|_| ParseError::InvalidLength)?
                .parse::<usize>()
                .map_err(|_| ParseError::InvalidLength)?;

            let body_end = body_start
                .checked_add(len)
                .ok_or(ParseError::InvalidLength)?;

            let used = body_end.checked_add(2).ok_or(ParseError::InvalidLength)?;

            let terminator = input.get(body_end..used).ok_or(ParseError::Incomplete)?;
            if terminator != b"\r\n" {
                return Err(ParseError::InvalidFormat);
            }

            let body = input[body_start..body_end].to_vec();
            Ok((RespValue::BulkString(Some(body)), used))
        }
        // Array
        b'*' => {
            let header_end  = find_crlf(input).ok_or(ParseError::Incomplete)?;
            let count_b = &input[1..header_end];
            if !count_b.iter().all(u8::is_ascii_digit) {
                return Err(ParseError::InvalidLength);
            }
            
            let count = std::str::from_utf8(count_b)
                .map_err(|_| ParseError::InvalidLength)?
                .parse::<usize>()
                .map_err(|_| ParseError::InvalidLength)?;


            let mut offset = header_end + 2;
            let mut items = Vec::new();

            for _ in 0..count {
                if offset == input.len() {
                    return Err(ParseError::Incomplete);
                }

                let (item, used) = decode(&input[offset..])?;
                items.push(item);
                offset += used;
            }

            Ok((RespValue::Array(items), offset))
        }
        _ =>{ 
            println!("error");
            Err(ParseError::InvalidPrefix)
        }}
}
