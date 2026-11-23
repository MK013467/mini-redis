#[cfg(test)]
mod tests {
    use crate::resp::{decode, RespValue};

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

    // Todos: after finishing other decoding methods
    fn decodes_array(){      

    }
}