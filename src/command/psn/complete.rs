use super::types::StatusConnection;
use super::urc::DataConnectionActivated;
use crate::command::Urc;
use atat::nom::{bytes, character, combinator, sequence};

pub(crate) fn parse_ip_status(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (_, status, ip, _))) = sequence::tuple::<_, _, (), _>((
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag(b"+MIPCALL:"),
            combinator::opt(bytes::complete::tag(b" ")),
        ))),
        character::streaming::one_of("012"),
        combinator::opt(sequence::preceded(
            bytes::streaming::tag(b","),
            combinator::recognize(sequence::tuple((
                character::streaming::u8,
                bytes::streaming::tag(b"."),
                character::streaming::u8,
                bytes::streaming::tag(b"."),
                character::streaming::u8,
                bytes::streaming::tag(b"."),
                character::streaming::u8,
            ))),
        )),
        bytes::complete::tag(b"\r\n"),
    ))(resp)
    {
        if reminder.is_empty() {
            match status {
                '0' => {
                    return Some(Urc::DataConnectionActivated(DataConnectionActivated {
                        sc: StatusConnection::Disconnect,
                    }));
                }
                '1' => {
                    if let Some(ip_u8) = ip {
                        if let Ok(ip_str) = core::str::from_utf8(ip_u8) {
                            if let Ok(ip_addr) = ip_str.parse() {
                                return Some(Urc::DataConnectionActivated(
                                    DataConnectionActivated {
                                        sc: StatusConnection::Connected(ip_addr),
                                    },
                                ));
                            }
                        }
                    }
                }
                '2' => {
                    return Some(Urc::DataConnectionActivated(DataConnectionActivated {
                        sc: StatusConnection::Busy,
                    }));
                }
                _ => (),
            }
        }
    }

    None
}

pub(crate) fn parse_ip_only(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (_, ip, _))) = sequence::tuple::<_, _, (), _>((
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag(b"+MIPCALL:"),
            combinator::opt(bytes::complete::tag(b" ")),
        ))),
        combinator::recognize(sequence::tuple((
            character::streaming::u8,
            bytes::streaming::tag(b"."),
            character::streaming::u8,
            bytes::streaming::tag(b"."),
            character::streaming::u8,
            bytes::streaming::tag(b"."),
            character::streaming::u8,
        ))),
        bytes::complete::tag(b"\r\n"),
    ))(resp)
    {
        if reminder.is_empty() {
            if let Ok(ip_str) = core::str::from_utf8(ip) {
                if let Ok(ip_addr) = ip_str.parse() {
                    return Some(Urc::DataConnectionActivated(DataConnectionActivated {
                        sc: StatusConnection::Connected(ip_addr),
                    }));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_nal::IpAddr;
    use no_std_net::Ipv4Addr;

    #[test]
    fn can_parse_ip_status_t1() {
        let res = parse_ip_status(b"+MIPCALL: 0\r\n").unwrap();

        if let Urc::DataConnectionActivated(dca) = res {
            let DataConnectionActivated { sc } = dca;

            assert_eq!(sc, StatusConnection::Disconnect);
        }
    }

    #[test]
    fn can_parse_ip_status_t2() {
        let res = parse_ip_status(b"+MIPCALL: 1,10.10.10.10\r\n").unwrap();

        if let Urc::DataConnectionActivated(dca) = res {
            let DataConnectionActivated { sc } = dca;

            assert_eq!(
                sc,
                StatusConnection::Connected(IpAddr::V4(Ipv4Addr::new(10, 10, 10, 10)))
            );
        }
    }

    #[test]
    fn can_parse_ip_status_t3() {
        let res = parse_ip_only(b"+MIPCALL: 10.10.10.10\r\n").unwrap();

        if let Urc::DataConnectionActivated(dca) = res {
            let DataConnectionActivated { sc } = dca;

            assert_eq!(
                sc,
                StatusConnection::Connected(IpAddr::V4(Ipv4Addr::new(10, 10, 10, 10)))
            );
        }
    }
}
