use atat::atat_derive::{AtatEnum, AtatLen};
use embedded_nal::IpAddr;
use heapless::String;
use serde::{Deserialize, Serialize};

/// Indicates the state of PDP context activation
#[derive(Clone, PartialEq, Eq, AtatEnum)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PDPContextStatus {
    /// 0: deactivated
    Deactivated = 0,
    /// 1: activated
    Activated = 1,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
#[at_arg(u8)]
pub enum PacketSwitchedParam {
    /// • 0: Protocol type; the allowed values of <param_val> parameter are
    #[at_arg(value = 0)]
    ProtocolType(ProtocolType),
    /// • 1: APN - <param_val> defines the APN text string, e.g.
    /// "apn.provider.com"; the maximum length is 99. The factory-programmed
    /// value is an empty string.
    #[at_arg(value = 1)]
    APN(String<99>),
    /// • 2: username - <param_val> is the user name text string for the
    /// authentication phase. The factory-programmed value is an empty string.
    #[at_arg(value = 2)]
    Username(String<64>),
    /// • 3: password - <param_val> is the password text string for the
    /// authentication phase. Note: the AT+UPSD read command with param_tag = 3
    /// is not allowed and the read all command does not display it
    #[at_arg(value = 3)]
    Password(String<64>),
    /// • 4: DNS1 - <param_val> is the text string of the primary DNS address.
    /// IPv4 DNS addresses are specified in dotted decimal notation form (i.e.
    /// four numbers in range 0-255 separated by periods, e.g.
    /// "xxx.yyy.zzz.www"). IPv6 DNS addresses are specified in standard IPv6
    /// notation form (2001:DB8:: address compression is allowed). The
    /// factory-programmed value is "0.0.0.0".
    #[at_arg(value = 4)]
    DNS1(#[at_arg(len = 45)] IpAddr),
    /// • 5: DNS2 - <param_val> is the text string of the secondary DNS address.
    /// IPv4 DNS addresses are specified in dotted decimal notation form (i.e.
    /// four numbers in range 0-255 separated by periods, e.g.
    /// "xxx.yyy.zzz.www"). IPv6 DNS addresses are specified in standard IPv6
    /// notation form (2001:DB8:: address compression is allowed). The
    /// factory-programmed value is "0.0.0.0".
    #[at_arg(value = 5)]
    DNS2(#[at_arg(len = 45)] IpAddr),
    /// • 6: authentication - the <param_val> parameter selects the
    /// authentication type:
    #[at_arg(value = 6)]
    Authentication(AuthenticationType),
    /// • 7: IP address - <param_val> is the text string of the static IP
    /// address given by the ISP in dotted decimal notation form (i.e. four
    /// numbers in range 0-255 separated by periods, e.g. "xxx.yyy.zzz.www").
    /// The factory-programmed value is "0.0.0.0". Note: IP address set as
    /// "0.0.0.0" means dynamic IP address assigned during PDP context
    /// activation
    #[at_arg(value = 7)]
    IPAddress(#[at_arg(len = 45)] IpAddr),
    /// • 8: data compression - the <param_val> parameter refers to the default
    /// parameter named d_comp and selects the data compression type:
    #[at_arg(value = 8)]
    DataCompression(DataCompression),
    /// • 9: header compression - the <param_val> parameter refers to the
    /// default parameter named h_comp and selects the header compression type:
    #[at_arg(value = 9)]
    HeaderCompression(HeaderCompression),
    /// • 10: QoS precedence - the <param_val> parameter selects the precedence
    /// class:
    #[at_arg(value = 10)]
    QoSPrecedence(QoSPrecedence),
    /// • 11: QoS delay - the <param_val> parameter selects the delay class:
    #[at_arg(value = 11)]
    QoSDelay(QoSDelay),
    /// • 12: QoS reliability - the <param_val> parameter selects the
    /// reliability class:
    #[at_arg(value = 12)]
    QoSReliability(QoSReliability),
    /// • 13: QoS peak rate - the <param_val> parameter selects the peak
    /// throughput in range 0-9. The factory-programmed value is 0. • 14: QoS
    /// mean rate - the <param_val> parameter selects the mean throughput in
    /// range 0-18, 31. The factory-programmed value is 0. • 15: minimum QoS
    /// precedence - the <param_val> parameter selects the acceptable value for
    /// the precedence class: o 0 (factory-programmed value): subscribed o 1:
    /// high o 2: normal o 3: low • 16: minimum QoS delay - the <param_val>
    /// parameter selects the acceptable value for the delay class: o 0
    /// (factory-programmed value): subscribed o 1: class 1 o 2: class 2 o 3:
    /// class 3 o 4: best effort • 17: minimum QoS reliability - the <param_val>
    /// parameter selects the minimum acceptable value for the reliability
    /// class: o 0 (factory-programmed value): subscribed o 1: class 1
    /// (Interpreted as class 2) o 2: class 2 (GTP Unack, LLC Ack and Protected,
    /// RLC Ack) o 3: class 3 (GTP Unack, LLC Unack and Protected, RLC Ack) o 4:
    /// class 4 (GTP Unack, LLC Unack and Protected, RLC Unack) o 5: class 5
    /// (GTP Unack, LLC Unack and Unprotected, RLC Unack) o 6: class 6
    /// (Interpreted as class 3) • 18: minimum QoS peak rate - the <param_val>
    /// parameter selects the acceptable value for the peak throughput in range
    /// 0-9. The factory-programmed value is 0. • 19: minimum QoS mean rate -
    /// the <param_val> parameter selects the acceptable value for the mean
    /// throughput in range 0-18, 31. The factory-programmed value is 0. • 20:
    /// 3G QoS delivery order - the <param_val> parameter selects the acceptable
    /// value for the delivery order: o 0 (factory-programmed value): subscribed
    /// o 1: enable o 2: disable • 21: 3G QoS erroneous SDU delivery - the
    /// <param_val> parameter selects the acceptable value for the erroneous SDU
    /// delivery: o 0 (factory-programmed value): subscribed o 1: no detection o
    /// 2: enable o 3: disable • 22: 3G QoS extended guaranteed downlink bit
    /// rate - <param_val> is the value for the extended guaranteed downlink bit
    /// rate in kb/s. The factory-programmed value is 0. • 23: 3G QoS extended
    /// maximum downlink bit rate - <param_val> is the value for the extended
    /// maximum downlink bit rate in kb/s. The factory-programmed value is 0. •
    /// 24: 3G QoS guaranteed downlink bit rate - <param_val> is the value for
    /// the guaranteed downlink bit rate in kb/s. The factory-programmed value
    /// is 0. • 25: 3G QoS guaranteed uplink bit rate - <param_val> is the value
    /// for the guaranteed uplink bit rate in kb/s. The factory-programmed value
    /// is 0. • 26: 3G QoS maximum downlink bit rate - <param_val> is the value
    /// for the maximum downlink bit rate in kb/s. The factory-programmed value
    /// is 0. • 27: 3G QoS maximum uplink bit rate - <param_val> is the value
    /// for the maximum uplink bit rate in kb/s. The factory-programmed value is
    /// 0. • 28: 3G QoS maximum SDU size - <param_val> is the value for the
    ///    maximum SDU size in octets. The factory-programmed value is 0. • 29:
    ///    3G QoS residual bit error rate - <param_val> selects the acceptable
    ///    value for the residual bit error rate: o 0 (factory-programmed
    ///    value): subscribed o 1: 5E2 o 2: 1E2 o 3: 5E3 o 4: 4E3 o 5: 1E3 o 6:
    ///    1E4 o 7: 1E5 o 8: 1E6 o 9: 6E8 • 30: 3G QoS SDU error ratio -
    ///    <param_val> selects the acceptable value for the SDU error ratio: o 0
    ///    (factory-programmed value): subscribed o 1: 1E2 o 2: 7E3 o 3: 1E3 o
    ///    4: 1E4 o 5: 1E5 o 6: 1E6 o 7: 1E1 • 31: 3G QoS signalling indicator -
    ///    <param_val> selects the acceptable value for the signalling
    ///    indicator: o 0 (factory-programmed value): subscribed o 1: signalling
    ///    indicator 1 • 32: 3G QoS source statistics descriptor - <param_val>
    ///    selects the acceptable value for the source statistics descriptor: o
    ///    0 (factory-programmed value): subscribed o 1: source statistics
    ///    descriptor 1 • 33: 3G QoS traffic class - <param_val> selects the
    ///    acceptable value for the traffic class: o 0 (factory-programmed
    ///    value): subscribed o 1: conversational o 2: streaming o 3:
    ///    interactive o 4: background • 34: 3G QoS traffic priority
    /// - <param_val> selects the acceptable value for the traffic priority: o 0
    ///   (factory-programmed value): subscribed o 1: priority 1 o 2: priority 2
    ///   o 3: priority 3 • 35: 3G QoS transfer delay - <param_val> is the value
    ///   for the transfer delay in milliseconds. The factory-programmed value
    ///   is 0. • 36: 3G minimum QoS delivery order - <param_val> selects the
    ///   acceptable value for the delivery order: o 0 (factory-programmed
    ///   value): subscribed o 1: enable o 2: disable • 37: 3G minimum QoS
    ///   erroneous SDU delivery - <param_val> selects the acceptable value for
    ///   the erroneous SDU delivery: o 0 (factory-programmed value): subscribed
    ///   o 1: no detection o 2: enable o 3: disable • 38: 3G minimum QoS
    ///   extended guaranteed downlink bit rate
    /// - <param_val> is the value for the extended guaranteed downlink bit rate
    ///   in kb/s. The factoryprogrammed value is 0. • 39: 3G minimum QoS
    ///   extended maximum downlink bit rate - <param_val> is the value for the
    ///   extended maximum downlink bit rate in kb/s. The factory-programmed
    ///   value is 0. • 40: 3G minimum QoS guaranteed downlink bit rate -
    ///   <param_val> is the value for the guaranteed downlink bit rate in kb/s.
    ///   The factory-programmed value is 0. • 41: 3G minimum QoS guaranteed
    ///   uplink bit rate - <param_val> is the value for the guaranteed uplink
    ///   bit rate in kb/s. The factory-programmed value is 0. • 42: 3G minimum
    ///   QoS maximum downlink bit rate - <param_val> is the value for the
    ///   maximum downlink bit rate in kb/s. The factory-programmed value is 0.
    ///   • 43: 3G minimum QoS maximum uplink bit rate - <param_val> is the
    ///   value for the maximum uplink bit rate in kb/s. The factory-programmed
    ///   value is 0. • 44: 3G minimum QoS maximum SDU size - <param_val> is the
    ///   value for the maximum SDU size in octets. The factory-programmed value
    ///   is 0. • 45: 3G minimum QoS residual bit error rate - <param_val>
    ///   selects the acceptable value for the residual bit error rate: o 0
    ///   (factory-programmed value): subscribed o 1: 5E2 o 2: 1E2 o 3: 5E3 o 4:
    ///   4E3 o 5: 1E3 o 6: 1E4 o 7: 1E5 o 8: 1E6 o 9: 6E8 • 46: 3G minimum QoS
    ///   SDU error ratio - <param_val> selects the acceptable value for the SDU
    ///   error ratio: o 0 (factory-programmed value): subscribed o 1: 1E2 o 2:
    ///   7E3 o 3: 1E3 o 4: 1E4 o 5: 1E5 o 6: 1E6 o 7: 1E1 • 47: 3G minimum QoS
    ///   signalling indicator
    /// - <param_val> selects the acceptable value for the signalling indicator:
    ///   o 0 (factory-programmed value): subscribed o 1: signalling indicator 1
    ///   • 48: 3G minimum QoS source statistics descriptor - <param_val>
    ///   selects the acceptable value for the source statistics descriptor: o 0
    ///   (factory-programmed value): subscribed o 1: source statistics
    ///   descriptor 1 • 49: 3G minimum QoS traffic class - <param_val> selects
    ///   the acceptable value for the traffic class: o 0 (factory-programmed
    ///   value): subscribed o 1: conversational o 2: streaming o 3: interactive
    ///   o 4: background • 50: 3G minimum QoS traffic priority - <param_val>
    ///   selects the acceptable value for the traffic priority: o 0
    ///   (factory-programmed value): subscribed o 1: priority 1 o 2: priority 2
    ///   o 3: priority 3 • 51: 3G Minimum QoS transfer delay - <param_val> is
    ///   the value for the transfer delay in milliseconds. The
    ///   factory-programmed value is 0. QoSDelay3G(u32),
    #[at_arg(value = 255)]
    UNUSED,
    /// • 100: map the +UPSD profile to the specified <cid> in the +CGDCONT
    /// table. o 0: map the current profile to default bearer PDP ID o 1: map
    /// the current profile to <cid> 1 o 2: map the current profile to <cid> 2 o
    /// 3: map the current profile to <cid> 3 o 4: map the current profile to
    /// <cid> 4 o 5: map the current profile to <cid> 5 o 6: map the current
    /// profile to <cid> 6 o 7: map the current profile to <cid> 7 o 8: map the
    /// current profile to <cid> 8 CurrentProfileMap(u8),
    #[at_arg(value = 100)]
    MapProfile(ContextId),
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum PacketSwitchedParamReq {
    /// • 0: Protocol type; the allowed values of <param_val> parameter are
    ProtocolType = 0,
    /// • 1: APN - <param_val> defines the APN text string, e.g.
    /// "apn.provider.com"; the maximum length is 99. The factory-programmed
    /// value is an empty string.
    APN = 1,
    /// • 2: username - <param_val> is the user name text string for the
    /// authentication phase. The factory-programmed value is an empty string.
    Username = 2,
    /// • 3: password - <param_val> is the password text string for the
    /// authentication phase. Note: the AT+UPSD read command with param_tag = 3
    /// is not allowed and the read all command does not display it
    Password = 3,
    /// • 4: DNS1 - <param_val> is the text string of the primary DNS address.
    /// IPv4 DNS addresses are specified in dotted decimal notation form (i.e.
    /// four numbers in range 0-255 separated by periods, e.g.
    /// "xxx.yyy.zzz.www"). IPv6 DNS addresses are specified in standard IPv6
    /// notation form (2001:DB8:: address compression is allowed). The
    /// factory-programmed value is "0.0.0.0".
    DNS1 = 4,
    /// • 5: DNS2 - <param_val> is the text string of the secondary DNS address.
    /// IPv4 DNS addresses are specified in dotted decimal notation form (i.e.
    /// four numbers in range 0-255 separated by periods, e.g.
    /// "xxx.yyy.zzz.www"). IPv6 DNS addresses are specified in standard IPv6
    /// notation form (2001:DB8:: address compression is allowed). The
    /// factory-programmed value is "0.0.0.0".
    DNS2 = 5,
    /// • 6: authentication - the <param_val> parameter selects the
    /// authentication type:
    Authentication = 6,
    /// • 7: IP address - <param_val> is the text string of the static IP
    /// address given by the ISP in dotted decimal notation form (i.e. four
    /// numbers in range 0-255 separated by periods, e.g. "xxx.yyy.zzz.www").
    /// The factory-programmed value is "0.0.0.0". Note: IP address set as
    /// "0.0.0.0" means dynamic IP address assigned during PDP context
    /// activation
    IPAddress = 7,
    /// • 8: data compression - the <param_val> parameter refers to the default
    /// parameter named d_comp and selects the data compression type:
    DataCompression = 8,
    /// • 9: header compression - the <param_val> parameter refers to the
    /// default parameter named h_comp and selects the header compression type:
    HeaderCompression = 9,
    /// • 10: QoS precedence - the <param_val> parameter selects the precedence
    /// class:
    QoSPrecedence = 10,
    /// • 11: QoS delay - the <param_val> parameter selects the delay class:
    QoSDelay = 11,
    /// • 12: QoS reliability - the <param_val> parameter selects the
    /// reliability class:
    QoSReliability = 12,
    /// • 13: QoS peak rate - the <param_val> parameter selects the peak
    /// throughput in range 0-9. The factory-programmed value is 0. • 14: QoS
    /// mean rate - the <param_val> parameter selects the mean throughput in
    /// range 0-18, 31. The factory-programmed value is 0. • 15: minimum QoS
    /// precedence - the <param_val> parameter selects the acceptable value for
    /// the precedence class: o 0 (factory-programmed value): subscribed o 1:
    /// high o 2: normal o 3: low • 16: minimum QoS delay - the <param_val>
    /// parameter selects the acceptable value for the delay class: o 0
    /// (factory-programmed value): subscribed o 1: class 1 o 2: class 2 o 3:
    /// class 3 o 4: best effort • 17: minimum QoS reliability - the <param_val>
    /// parameter selects the minimum acceptable value for the reliability
    /// class: o 0 (factory-programmed value): subscribed o 1: class 1
    /// (Interpreted as class 2) o 2: class 2 (GTP Unack, LLC Ack and Protected,
    /// RLC Ack) o 3: class 3 (GTP Unack, LLC Unack and Protected, RLC Ack) o 4:
    /// class 4 (GTP Unack, LLC Unack and Protected, RLC Unack) o 5: class 5
    /// (GTP Unack, LLC Unack and Unprotected, RLC Unack) o 6: class 6
    /// (Interpreted as class 3) • 18: minimum QoS peak rate - the <param_val>
    /// parameter selects the acceptable value for the peak throughput in range
    /// 0-9. The factory-programmed value is 0. • 19: minimum QoS mean rate -
    /// the <param_val> parameter selects the acceptable value for the mean
    /// throughput in range 0-18, 31. The factory-programmed value is 0. • 20:
    /// 3G QoS delivery order - the <param_val> parameter selects the acceptable
    /// value for the delivery order: o 0 (factory-programmed value): subscribed
    /// o 1: enable o 2: disable • 21: 3G QoS erroneous SDU delivery - the
    /// <param_val> parameter selects the acceptable value for the erroneous SDU
    /// delivery: o 0 (factory-programmed value): subscribed o 1: no detection o
    /// 2: enable o 3: disable • 22: 3G QoS extended guaranteed downlink bit
    /// rate - <param_val> is the value for the extended guaranteed downlink bit
    /// rate in kb/s. The factory-programmed value is 0. • 23: 3G QoS extended
    /// maximum downlink bit rate - <param_val> is the value for the extended
    /// maximum downlink bit rate in kb/s. The factory-programmed value is 0. •
    /// 24: 3G QoS guaranteed downlink bit rate - <param_val> is the value for
    /// the guaranteed downlink bit rate in kb/s. The factory-programmed value
    /// is 0. • 25: 3G QoS guaranteed uplink bit rate - <param_val> is the value
    /// for the guaranteed uplink bit rate in kb/s. The factory-programmed value
    /// is 0. • 26: 3G QoS maximum downlink bit rate - <param_val> is the value
    /// for the maximum downlink bit rate in kb/s. The factory-programmed value
    /// is 0. • 27: 3G QoS maximum uplink bit rate - <param_val> is the value
    /// for the maximum uplink bit rate in kb/s. The factory-programmed value is
    /// 0. • 28: 3G QoS maximum SDU size - <param_val> is the value for the
    ///    maximum SDU size in octets. The factory-programmed value is 0. • 29:
    ///    3G QoS residual bit error rate - <param_val> selects the acceptable
    ///    value for the residual bit error rate: o 0 (factory-programmed
    ///    value): subscribed o 1: 5E2 o 2: 1E2 o 3: 5E3 o 4: 4E3 o 5: 1E3 o 6:
    ///    1E4 o 7: 1E5 o 8: 1E6 o 9: 6E8 • 30: 3G QoS SDU error ratio -
    ///    <param_val> selects the acceptable value for the SDU error ratio: o 0
    ///    (factory-programmed value): subscribed o 1: 1E2 o 2: 7E3 o 3: 1E3 o
    ///    4: 1E4 o 5: 1E5 o 6: 1E6 o 7: 1E1 • 31: 3G QoS signalling indicator -
    ///    <param_val> selects the acceptable value for the signalling
    ///    indicator: o 0 (factory-programmed value): subscribed o 1: signalling
    ///    indicator 1 • 32: 3G QoS source statistics descriptor - <param_val>
    ///    selects the acceptable value for the source statistics descriptor: o
    ///    0 (factory-programmed value): subscribed o 1: source statistics
    ///    descriptor 1 • 33: 3G QoS traffic class - <param_val> selects the
    ///    acceptable value for the traffic class: o 0 (factory-programmed
    ///    value): subscribed o 1: conversational o 2: streaming o 3:
    ///    interactive o 4: background • 34: 3G QoS traffic priority
    /// - <param_val> selects the acceptable value for the traffic priority: o 0
    ///   (factory-programmed value): subscribed o 1: priority 1 o 2: priority 2
    ///   o 3: priority 3 • 35: 3G QoS transfer delay - <param_val> is the value
    ///   for the transfer delay in milliseconds. The factory-programmed value
    ///   is 0. • 36: 3G minimum QoS delivery order - <param_val> selects the
    ///   acceptable value for the delivery order: o 0 (factory-programmed
    ///   value): subscribed o 1: enable o 2: disable • 37: 3G minimum QoS
    ///   erroneous SDU delivery - <param_val> selects the acceptable value for
    ///   the erroneous SDU delivery: o 0 (factory-programmed value): subscribed
    ///   o 1: no detection o 2: enable o 3: disable • 38: 3G minimum QoS
    ///   extended guaranteed downlink bit rate
    /// - <param_val> is the value for the extended guaranteed downlink bit rate
    ///   in kb/s. The factoryprogrammed value is 0. • 39: 3G minimum QoS
    ///   extended maximum downlink bit rate - <param_val> is the value for the
    ///   extended maximum downlink bit rate in kb/s. The factory-programmed
    ///   value is 0. • 40: 3G minimum QoS guaranteed downlink bit rate -
    ///   <param_val> is the value for the guaranteed downlink bit rate in kb/s.
    ///   The factory-programmed value is 0. • 41: 3G minimum QoS guaranteed
    ///   uplink bit rate - <param_val> is the value for the guaranteed uplink
    ///   bit rate in kb/s. The factory-programmed value is 0. • 42: 3G minimum
    ///   QoS maximum downlink bit rate - <param_val> is the value for the
    ///   maximum downlink bit rate in kb/s. The factory-programmed value is 0.
    ///   • 43: 3G minimum QoS maximum uplink bit rate - <param_val> is the
    ///   value for the maximum uplink bit rate in kb/s. The factory-programmed
    ///   value is 0. • 44: 3G minimum QoS maximum SDU size - <param_val> is the
    ///   value for the maximum SDU size in octets. The factory-programmed value
    ///   is 0. • 45: 3G minimum QoS residual bit error rate - <param_val>
    ///   selects the acceptable value for the residual bit error rate: o 0
    ///   (factory-programmed value): subscribed o 1: 5E2 o 2: 1E2 o 3: 5E3 o 4:
    ///   4E3 o 5: 1E3 o 6: 1E4 o 7: 1E5 o 8: 1E6 o 9: 6E8 • 46: 3G minimum QoS
    ///   SDU error ratio - <param_val> selects the acceptable value for the SDU
    ///   error ratio: o 0 (factory-programmed value): subscribed o 1: 1E2 o 2:
    ///   7E3 o 3: 1E3 o 4: 1E4 o 5: 1E5 o 6: 1E6 o 7: 1E1 • 47: 3G minimum QoS
    ///   signalling indicator
    /// - <param_val> selects the acceptable value for the signalling indicator:
    ///   o 0 (factory-programmed value): subscribed o 1: signalling indicator 1
    ///   • 48: 3G minimum QoS source statistics descriptor - <param_val>
    ///   selects the acceptable value for the source statistics descriptor: o 0
    ///   (factory-programmed value): subscribed o 1: source statistics
    ///   descriptor 1 • 49: 3G minimum QoS traffic class - <param_val> selects
    ///   the acceptable value for the traffic class: o 0 (factory-programmed
    ///   value): subscribed o 1: conversational o 2: streaming o 3: interactive
    ///   o 4: background • 50: 3G minimum QoS traffic priority - <param_val>
    ///   selects the acceptable value for the traffic priority: o 0
    ///   (factory-programmed value): subscribed o 1: priority 1 o 2: priority 2
    ///   o 3: priority 3 • 51: 3G Minimum QoS transfer delay - <param_val> is
    ///   the value for the transfer delay in milliseconds. The
    ///   factory-programmed value is 0. QoSDelay3G(u32),
    UNUSED = 255,
    /// • 100: map the +UPSD profile to the specified <cid> in the +CGDCONT
    /// table. o 0: map the current profile to default bearer PDP ID o 1: map
    /// the current profile to <cid> 1 o 2: map the current profile to <cid> 2 o
    /// 3: map the current profile to <cid> 3 o 4: map the current profile to
    /// <cid> 4 o 5: map the current profile to <cid> 5 o 6: map the current
    /// profile to <cid> 6 o 7: map the current profile to <cid> 7 o 8: map the
    /// current profile to <cid> 8 CurrentProfileMap(u8),
    MapProfile = 100,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum ProtocolType {
    /// (factory-programmed value): IPv4
    IPv4 = 0,
    /// IPv6
    IPv6 = 1,
    /// IPv4v6 with IPv4 preferred for internal sockets
    IPv4v6PreferV4Internal = 2,
    /// IPv4v6 with IPv6 preferred for internal sockets
    IPv4v6PreferV6Internal = 3,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum AuthenticationType {
    /// (factory-programmed value): none
    None = 0,
    /// PAP
    PAP = 1,
    /// CHAP
    CHAP = 2,
    /// automatic selection of authentication type (none/CHAP/PAP)
    Auto = 3,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum DataCompression {
    /// (factory-programmed value): off
    Off = 0,
    /// predefined, i.e. V.42bis
    Predefined = 1,
    /// V.42bis
    V42Bits = 2,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum HeaderCompression {
    /// (factory-programmed value): off
    Off = 0,
    /// predefined, i.e. RFC1144
    Predefined = 1,
    /// RFC1144
    RFC1144 = 2,
    /// RFC2507
    RFC2507 = 3,
    /// RFC3095
    RFC3095 = 4,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum QoSPrecedence {
    /// (factory-programmed value): subscribed
    Subscribed = 0,
    /// high
    High = 1,
    /// normal
    Normal = 2,
    /// low
    Low = 3,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum QoSDelay {
    /// (factory-programmed value): subscribed
    Subscribed = 0,
    /// class 1
    Class1 = 1,
    /// class 2
    Class2 = 2,
    /// class 3
    Class3 = 3,
    /// best effort
    BestEffort = 4,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum QoSReliability {
    /// (factory-programmed value): subscribed
    Subscribed = 0,
    /// class 1 (Interpreted as class 2)
    Class1 = 1,
    /// class 2 (GTP Unack, LLC Ack and Protected, RLC Ack)
    Class2 = 2,
    /// class 3 (GTP Unack, LLC Unack and Protected, RLC Ack)
    Class3 = 3,
    /// class 4 (GTP Unack, LLC Unack and Protected, RLC Unack)
    Class4 = 4,
    /// class 5 (GTP Unack, LLC Unack and Unprotected, RLC Unack)
    Class5 = 5,
    /// class 6 (Interpreted as class 3)
    Class6 = 6,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum PacketSwitchedAction {
    /// • 0: It clears the specified profile resetting all the parameters to their
    /// factory programmed values
    Reset = 0,
    /// • 1: It saves all the parameters in NVM
    Store = 1,
    /// • 2: It reads all the parameters from NVM
    Load = 2,
    /// • 3: It activates a PDP context with the specified profile, using the current
    /// parameters
    Activate = 3,
    /// • 4: It deactivates the PDP context associated with the specified profile
    Deactivate = 4,
}

#[derive(Debug, Clone, PartialEq, Eq, AtatEnum)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GPRSAttachedState {
    /// • 0: Detached
    Detached = 0,
    /// • 1: Attached
    Attached = 1,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum GPRSNetworkRegistrationUrcConfig {
    /// • 0 (default value and factory-programmed value): network registration
    /// URC disabled
    UrcDisabled = 0,
    /// • 1: network registration URC enabled
    UrcEnabled = 1,
    /// • 2: network registration and location information URC enabled
    UrcVerbose = 2,
}

/// Controls the processing of URCs specified within this command.
#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum PSEventReportingMode {
    /// • 0 (default value): buffer URCs in the MT; if the buffer is full the
    /// oldest ones will be discarded
    CircularBufferUrcs = 0,
    /// • 1: discard URCs when V.24 link is reserved (online); otherwise forward
    /// them directly to the DTE
    DiscardUrcs = 1,
    /// • 2: buffer URCs in the MT when link reserved (online) and flush them to
    /// the DTE when the link becomes available; otherwise forward them directly
    /// to the DTE
    BufferUrcs = 2,
}

#[derive(Debug, Clone, PartialEq, Eq, AtatEnum)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GPRSNetworkRegistrationStat {
    /// • 0: not registered, the MT is not currently searching an operator to
    /// register to
    NotRegistered = 0,
    /// • 1: registered, home network
    Registered = 1,
    /// • 2: not registered, but MT is currently searching a new operator to
    /// register to
    NotRegisteredSearching = 2,
    /// • 3: registration denied
    RegistrationDenied = 3,
    /// • 4: unknown (e.g. out of GERAN/UTRAN coverage)
    Unknown = 4,
    /// • 5: registered, roaming
    RegisteredRoaming = 5,
    /// • 8: attached for emergency bearer services only (see 3GPP TS 24.008
    /// [12] and 3GPP TS 24.301 [69] that specify the condition when the MS is
    /// considered as attached for emergency bearer services) (applicable only
    /// when <AcT> indicates 2,4,5,6)
    AttachedEmergencyOnly = 8,
}
#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum ExtendedPSNetworkRegistrationUrcConfig {
    /// • 0: network registration attach status URC disabled
    UrcDisabled = 0,
    /// • 1: network registration attach status URC +UREG enabled
    UrcEnabled = 1,
}

#[derive(Debug, Clone, PartialEq, Eq, AtatEnum)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ExtendedPSNetworkRegistrationState {
    /// • 0: not registered for PS service
    NotRegistered = 0,
    /// • 1: registered for PS service, RAT=2G, GPRS available
    Registered2gGprs = 1,
    /// • 2: registered for PS service, RAT=2G, EDGE available
    Registered2gEdge = 2,
    /// • 3: registered for PS service, RAT=3G, WCDMA available
    Registered3gWcdma = 3,
    /// • 4: registered for PS service, RAT=3G, HSDPA available
    Registered3gHsdpa = 4,
    /// • 5: registered for PS service, RAT=3G, HSUPA available
    Registered3gHsupa = 5,
    /// • 6: registered for PS service, RAT=3G, HSDPA and HSUPA available
    Registered3gHspdaHsupa = 6,
    /// • 7: registered for PS service, RAT=4G
    Registered4g = 7,
    /// • 8: registered for PS service, RAT=2G, GPRS available, DTM available
    Registered2gGprsDtm = 8,
    /// • 9: registered for PS service, RAT=2G, EDGE available, DTM available
    Registered2gEdgeDtm = 9,
}

/// Mode configuration
#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum EPSNetworkRegistrationUrcConfig {
    /// • 0: network registration URC disabled
    UrcDisabled = 0,
    /// • 1: network registration URC +CEREG: <stat> enabled
    UrcEnabled = 1,
    /// • 2: network registration and location information URC +CEREG:
    /// <stat>[,[<tac>],[<ci>],[<AcT>]] enabled
    UrcVerbose = 2,
    // • 3: network registration, location information and EMM cause value
    // information URC +CEREG:
    // <stat>[,[<tac>],[<ci>],[<AcT>][,<cause_type>,<reject_cause>]] enabled •
    // 4: PSM, network registration and location information information URC
    // +CEREG:
    // <stat>[,[<tac>],[<ci>],[<AcT>][,,[,[<Assigned_Active_Time>[,<Assigned_Periodic_TAU>]]]]]
    // enabled • 5: PSM, network registration, location information and EMM
    // cause value information URC +CEREG:
    // <stat>[,[<tac>],[<ci>],[<AcT>][,[<cause_type>],[<reject_cause>][,[<Assigned_Active_Time>,[<Assigned_Periodic_TAU>]]]]]
    // enabled
}

/// EPS registration status
#[derive(Debug, Clone, PartialEq, Eq, AtatEnum)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EPSNetworkRegistrationStat {
    /// • 0: not registered
    NotRegistered = 0,
    /// • 1: registered, home network
    Registered = 1,
    /// • 2: not registered, but the MT is currently trying to attach or
    /// searching an operator to register to
    NotRegisteredSearching = 2,
    /// • 3: registration denied
    RegistrationDenied = 3,
    /// • 4: unknown (e.g. out of E-UTRAN coverage)
    Unknown = 4,
    /// • 5: registered, roaming
    RegisteredRoaming = 5,
    /// • 8: attached for emergency bearer services only (see 3GPP TS 24.008
    /// [12] and 3GPP TS 24.301 [69] that specify the condition when the MS is
    /// considered as attached for emergency bearer services)
    AttachedEmergencyOnly = 8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, AtatLen)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ProfileId(pub u8);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, AtatLen)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ContextId(pub u8);
