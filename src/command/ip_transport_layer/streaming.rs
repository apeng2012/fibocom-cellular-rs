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
}
