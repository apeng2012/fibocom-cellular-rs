use super::urc::{CanSocketOpen, SocketReadData};
use crate::command::Urc;
use atat::nom::{bytes, character, combinator, sequence};
use heapless::Vec;
use ublox_sockets::PeerHandle;

pub(crate) fn parse_read_data(resp: &[u8]) -> Option<Urc> {
    if let Ok((_reminder, (_, id, _, (_, data)))) = sequence::tuple::<_, _, (), _>((
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag(b"+MIPRTCP:"),
            combinator::opt(bytes::complete::tag(b" ")),
        ))),
        character::complete::u8,
        bytes::complete::tag(","),
        combinator::flat_map(character::complete::u16, |data_len| {
            sequence::tuple((bytes::complete::tag(","), bytes::complete::take(data_len)))
        }),
    ))(resp)
    {
        return Vec::from_slice(data)
            .ok()
            .map(|vec| SocketReadData {
                id: PeerHandle(id),
                length: data.len(),
                data: vec,
            })
            .map(Urc::SocketReadData);
    }

    None
}

pub(crate) fn parse_can_socket_open(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (_, n1, on2, on3, on4, on5, on6))) = sequence::tuple::<_, _, (), _>((
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag(b"+MIPOPEN:"),
            combinator::opt(bytes::complete::tag(b" ")),
        ))),
        character::streaming::one_of("123456"),
        combinator::opt(combinator::flat_map(bytes::streaming::tag(b","), |_| {
            character::streaming::one_of("23456")
        })),
        combinator::opt(combinator::flat_map(bytes::streaming::tag(b","), |_| {
            character::streaming::one_of("3456")
        })),
        combinator::opt(combinator::flat_map(bytes::streaming::tag(b","), |_| {
            character::streaming::one_of("456")
        })),
        combinator::opt(combinator::flat_map(bytes::streaming::tag(b","), |_| {
            character::streaming::one_of("56")
        })),
        combinator::opt(combinator::flat_map(bytes::streaming::tag(b","), |_| {
            character::streaming::one_of("6")
        })),
    ))(resp)
    {
        if reminder.is_empty() {
            let mut v: Vec<PeerHandle, 6> = Vec::new();

            let _ = match n1.to_digit(10).map(|n| PeerHandle(n as u8)) {
                Some(id) => v.push(id),
                None => return None,
            };
            let _ = match on2.and_then(|c| c.to_digit(10).map(|d| PeerHandle(d as u8))) {
                Some(id) => v.push(id),
                None => return Some(Urc::CanSocketOpen(CanSocketOpen { id_list: v })),
            };
            let _ = match on3.and_then(|c| c.to_digit(10).map(|d| PeerHandle(d as u8))) {
                Some(id) => v.push(id),
                None => return Some(Urc::CanSocketOpen(CanSocketOpen { id_list: v })),
            };
            let _ = match on4.and_then(|c| c.to_digit(10).map(|d| PeerHandle(d as u8))) {
                Some(id) => v.push(id),
                None => return Some(Urc::CanSocketOpen(CanSocketOpen { id_list: v })),
            };
            let _ = match on5.and_then(|c| c.to_digit(10).map(|d| PeerHandle(d as u8))) {
                Some(id) => v.push(id),
                None => return Some(Urc::CanSocketOpen(CanSocketOpen { id_list: v })),
            };
            let _ = match on6.and_then(|c| c.to_digit(10).map(|d| PeerHandle(d as u8))) {
                Some(id) => v.push(id),
                None => return Some(Urc::CanSocketOpen(CanSocketOpen { id_list: v })),
            };

            return Some(Urc::CanSocketOpen(CanSocketOpen { id_list: v }));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_can_socket_open() {
        let res = parse_can_socket_open(b"+MIPOPEN: 1,2,3,4,5,6").unwrap();

        if let Urc::CanSocketOpen(sco) = res {
            let CanSocketOpen { id_list } = sco;

            let mut v: Vec<PeerHandle, 6> = Vec::new();
            v.push(PeerHandle(1)).ok();
            v.push(PeerHandle(2)).ok();
            v.push(PeerHandle(3)).ok();
            v.push(PeerHandle(4)).ok();
            v.push(PeerHandle(5)).ok();
            v.push(PeerHandle(6)).ok();

            assert_eq!(v, id_list);
        }
    }
}
