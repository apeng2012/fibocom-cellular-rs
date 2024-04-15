use atat::nom::{bytes, character, combinator, error::ParseError, sequence, IResult};

/// Matches the equivalent of regex: \r\n+MIPRTCP: [0-9],[0-9]+,[0-9]+\r\n
pub fn parse_read_data<'a, Error: ParseError<&'a [u8]>>(
    buf: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8], usize), Error> {
    let (reminder, (_, frame)) = sequence::tuple((
        bytes::streaming::tag("\r\n"),
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag(b"+MIPRTCP:"),
            combinator::opt(bytes::streaming::tag(b" ")),
            character::streaming::u8,
            bytes::streaming::tag(","),
            combinator::flat_map(character::streaming::u16, |data_len| {
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag(","),
                    bytes::streaming::take(data_len),
                )))
            }),
        ))),
    ))(buf)?;

    Ok((reminder, (frame, 2 + frame.len())))
}

/// Matches the equivalent of regex: \r\n+MIPOPEN: [1-6],[2-6],[3-6]+\r\n
pub fn parse_can_socket_open<'a, Error: ParseError<&'a [u8]>>(
    buf: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8], usize), Error> {
    let (reminder, (_, frame)) = sequence::tuple((
        bytes::streaming::tag("\r\n"),
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag(b"+MIPOPEN:"),
            combinator::opt(bytes::streaming::tag(b" ")),
            character::streaming::one_of("123456"), // 1
            combinator::opt(sequence::tuple((
                bytes::streaming::tag(b","),
                character::streaming::one_of("23456"), // 2
                combinator::opt(sequence::tuple((
                    bytes::streaming::tag(b","),
                    character::streaming::one_of("3456"), // 3
                    combinator::opt(sequence::tuple((
                        bytes::streaming::tag(b","),
                        character::streaming::one_of("456"), // 4
                        combinator::opt(sequence::tuple((
                            bytes::streaming::tag(b","),
                            character::streaming::one_of("56"), // 5
                            combinator::opt(sequence::tuple((
                                bytes::streaming::tag(b","),
                                character::streaming::one_of("6"), // 6
                            ))),
                        ))),
                    ))),
                ))),
            ))),
            bytes::streaming::tag("\r\n"),
        ))),
    ))(buf)?;

    Ok((reminder, (frame, 2 + frame.len())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_read_data() {
        let (reminder, result) =
            parse_read_data::<()>(b"\r\n+MIPRTCP: 5,8,HTTP\r\n\r\nTAIL").unwrap();
        assert_eq!(b"TAIL", reminder);
        assert_eq!(b"+MIPRTCP: 5,8,HTTP\r\n\r\n", result.0);
        assert_eq!(24, result.1);
    }

    #[test]
    fn can_parse_can_socket_open_t1() {
        let (_reminder, result) = parse_can_socket_open::<()>(b"\r\n+MIPOPEN: 6\r\n").unwrap();
        assert_eq!(b"+MIPOPEN: 6\r\n", result.0);
    }

    #[test]
    fn can_parse_can_socket_open_t2() {
        let (_reminder, result) = parse_can_socket_open::<()>(b"\r\n+MIPOPEN: 5,6\r\n").unwrap();
        assert_eq!(b"+MIPOPEN: 5,6\r\n", result.0);
    }

    #[test]
    fn can_parse_can_socket_open_t3() {
        let (_reminder, result) =
            parse_can_socket_open::<()>(b"\r\n+MIPOPEN: 1,2,3,4,5,6\r\n").unwrap();
        assert_eq!(b"+MIPOPEN: 1,2,3,4,5,6\r\n", result.0);
    }

    #[test]
    fn can_parse_can_socket_open_t4() {
        let res = parse_can_socket_open::<()>(b"\r\n+MIPOPEN: 1,1\r\n");
        assert!(res.is_err());
    }

    #[test]
    fn can_parse_can_socket_open_t5() {
        let res = parse_can_socket_open::<()>(b"\r\n+MIPOPEN: 1,0\r\n");
        assert!(res.is_err());
    }
}
