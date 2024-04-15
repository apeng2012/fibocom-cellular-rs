use atat::nom::{bytes, character, combinator, error::ParseError, sequence, IResult};

/// Matches the equivalent of regex: \r\n+MIPCALL: [0-2]\r\n
pub fn parse_ip_status<'a, Error: ParseError<&'a [u8]>>(
    buf: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8], usize), Error> {
    let (reminder, (_, frame)) = sequence::tuple((
        bytes::streaming::tag("\r\n"),
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag(b"+MIPCALL:"),
            combinator::opt(bytes::streaming::tag(b" ")),
            character::streaming::one_of("012"),
            combinator::opt(sequence::tuple((
                bytes::streaming::tag(b","),
                sequence::tuple((
                    character::streaming::u8,
                    bytes::streaming::tag(b"."),
                    character::streaming::u8,
                    bytes::streaming::tag(b"."),
                    character::streaming::u8,
                    bytes::streaming::tag(b"."),
                    character::streaming::u8,
                )),
            ))),
            bytes::streaming::tag("\r\n"),
        ))),
    ))(buf)?;

    Ok((reminder, (frame, 2 + frame.len())))
}

pub fn parse_ip_only<'a, Error: ParseError<&'a [u8]>>(
    buf: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8], usize), Error> {
    let (reminder, (_, frame)) = sequence::tuple((
        bytes::streaming::tag("\r\n"),
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag(b"+MIPCALL:"),
            combinator::opt(bytes::streaming::tag(b" ")),
            sequence::tuple((
                character::streaming::u8,
                bytes::streaming::tag(b"."),
                character::streaming::u8,
                bytes::streaming::tag(b"."),
                character::streaming::u8,
                bytes::streaming::tag(b"."),
                character::streaming::u8,
            )),
            bytes::streaming::tag("\r\n"),
        ))),
    ))(buf)?;

    Ok((reminder, (frame, 2 + frame.len())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_ip_status_t1() {
        let (_reminder, result) = parse_ip_status::<()>(b"\r\n+MIPCALL: 0\r\n").unwrap();
        assert_eq!(b"+MIPCALL: 0\r\n", result.0);
    }

    #[test]
    fn can_parse_ip_status_t2() {
        let (_reminder, result) =
            parse_ip_status::<()>(b"\r\n+MIPCALL: 1,10.10.10.10\r\n").unwrap();
        assert_eq!(b"+MIPCALL: 1,10.10.10.10\r\n", result.0);
    }

    #[test]
    fn can_parse_ip_status_t3() {
        let (_reminder, result) = parse_ip_only::<()>(b"\r\n+MIPCALL: 10.10.10.10\r\n").unwrap();
        assert_eq!(b"+MIPCALL: 10.10.10.10\r\n", result.0);
    }
}
