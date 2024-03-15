use super::urc::SocketReadData;
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
