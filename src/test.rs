#[cfg(test)]
mod tests {
    use crate::resp::{ParseError, RespValue, decode};

    #[test]
    /*
    covered cases:
    1) Typical
    2) read body in number
    3) empty body
     */
    fn decodes_bulk_strings() {
        let cases: &[(&[u8], &[u8], usize)] = &[
            (b"$3\r\nGET\r\n+OK\r\n", b"GET", 9),
            (b"$4\r\n1234\r\n", b"1234", 10),
            (b"$0\r\n\r\n", b"", 6),
        ];

        for &(input, expected_body, expected_used) in cases {
            let (value, used) = decode(input).unwrap();

            assert!(
                matches!(value, RespValue::BulkString(Some(body)) if body == expected_body),
                "input: {input:?}"
            );
            assert_eq!(used, expected_used, "input: {input:?}");
        }
    }

    #[test]
    fn decodes_integers() {
        let cases: &[(&[u8], i64, usize)] = &[
            (b":+64\r\n", 64, 6),
            (b":64\r\n", 64, 5),
            (b":-64\r\n", -64, 6),
        ];

        for &(input, expected_num, expected_used) in cases {
            let (value, used) = decode(input).unwrap();

            let RespValue::Integer(number) = value else {
                panic!("Integer가 반환되어야 함");
            };
            assert_eq!(number, expected_num);
            assert_eq!(used, expected_used, "input: {input:?}");
        }
    }

    #[test]
    fn decodes_mixed_array() {
        let input = b"*3\r\n+OK\r\n:42\r\n$3\r\nGET\r\n+NEXT\r\n";
        let (value, used) = decode(input).unwrap();
        let RespValue::Array(items) = value else {
            panic!("Array가 반환되어야 함");
        };

        assert!(matches!(
            items.as_slice(),
            [RespValue::SimpleString(text), RespValue::Integer(42), RespValue::BulkString(Some(body))]
                if text == "OK" && body == b"GET"
        ));
        assert_eq!(used, 23);
    }

    #[test]
    fn decodes_nested_array() {
        let input = b"*2\r\n*1\r\n:42\r\n:-1\r\n";
        let (value, used) = decode(input).unwrap();
        let RespValue::Array(items) = value else {
            panic!("Array가 반환되어야 함");
        };

        assert!(matches!(
            items.as_slice(),
            [RespValue::Array(nested), RespValue::Integer(-1)]
                if matches!(nested.as_slice(), [RespValue::Integer(42)])
        ));
        assert_eq!(used, input.len());
    }

    #[test]
    fn responds_to_ping_and_echo() {
        let cases: &[(&[u8], &[u8])] = &[
            (b"*1\r\n$4\r\nPING\r\n", b"+PONG\r\n"),
            (b"*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n", b"$5\r\nhello\r\n"),
        ];

        for &(request, expected_response) in cases {
            let (value, used) = decode(request).unwrap();

            assert_eq!(used, request.len(), "request: {request:?}");
            assert_eq!(
                crate::handle_command(&value).as_slice(),
                expected_response,
                "request: {request:?}"
            );
        }
    }
}
