//! Netlink parsing module

use crate::utils::network::netlink::{
    constants::{NLA_TYPE_MASK, NLMSG_DONE},
    types::{IfInfoMsg, NlMsgHdr, RtAttr},
};

use super::constants::{NLM_F_MULTI, NLMSG_ERROR};
use super::types::NlMsgErr;

pub const NLMSG_ALIGNTO: usize = 4;
pub const RTA_ALIGNTO: usize = 4;

#[inline]
pub const fn nlmsg_align(len: usize) -> usize {
    (len + NLMSG_ALIGNTO - 1) & !(NLMSG_ALIGNTO - 1)
}

#[inline]
pub const fn rta_align(len: usize) -> usize {
    (len + RTA_ALIGNTO - 1) & !(RTA_ALIGNTO - 1)
}

#[allow(unused)]
#[inline]
pub fn nlmsg_payload_len(hdr: &NlMsgHdr) -> usize {
    hdr.nlmsg_len as usize - NlMsgHdr::SIZE
}

#[allow(unused)]
#[inline]
pub fn rta_payload_len(attr: &RtAttr) -> usize {
    attr.rta_len as usize - RtAttr::SIZE
}

/// reads a nlmsghdr from a byte buffer
#[allow(unused)]
pub fn parse_nlmsg_header(buf: &[u8]) -> Option<NlMsgHdr> {
    if buf.len() < NlMsgHdr::SIZE {
        return None;
    }
    Some(unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const NlMsgHdr) })
}

/// reads an ifinfomsg from a RTM_NEWLINK payload
pub fn parse_ifinfomsg(payload: &[u8]) -> Option<(IfInfoMsg, &[u8])> {
    if payload.len() < IfInfoMsg::SIZE {
        return None;
    }
    let hdr = unsafe { core::ptr::read_unaligned(payload.as_ptr() as *const IfInfoMsg) };
    Some((hdr, &payload[IfInfoMsg::SIZE..]))
}

/// iterates over nlmsghdr messages in a buffer
pub struct NlMsgIter<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> NlMsgIter<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

impl<'a> Iterator for NlMsgIter<'a> {
    type Item = (u16, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + NlMsgHdr::SIZE > self.buf.len() {
            return None;
        }

        let hdr: NlMsgHdr =
            unsafe { core::ptr::read_unaligned(self.buf[self.pos..].as_ptr() as *const NlMsgHdr) };

        if hdr.nlmsg_type == NLMSG_DONE {
            return None;
        }

        let msg_len = hdr.nlmsg_len as usize;
        if msg_len < NlMsgHdr::SIZE || self.pos + msg_len > self.buf.len() {
            return None;
        }

        let payload = &self.buf[self.pos + NlMsgHdr::SIZE..self.pos + msg_len];
        self.pos += nlmsg_align(msg_len);

        Some((hdr.nlmsg_type, payload))
    }
}

/// iterates over rtattr entries in a byte slice
pub struct RtAttrIter<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> RtAttrIter<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

impl<'a> Iterator for RtAttrIter<'a> {
    type Item = (u16, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + RtAttr::SIZE > self.buf.len() {
            return None;
        }

        let attr: RtAttr =
            unsafe { core::ptr::read_unaligned(self.buf[self.pos..].as_ptr() as *const RtAttr) };

        let attr_len = attr.rta_len as usize;
        if attr_len < RtAttr::SIZE || self.pos + attr_len > self.buf.len() {
            return None;
        }

        let data = &self.buf[self.pos + RtAttr::SIZE..self.pos + attr_len];
        self.pos += rta_align(attr_len);

        Some((attr.rta_type & NLA_TYPE_MASK, data))
    }
}

/// returns the error code if the buffer contains a nlmsgerr structure
pub fn parse_nlmsg_error(buf: &[u8]) -> Option<i32> {
    if buf.len() < NlMsgHdr::SIZE {
        return None;
    }

    let hdr: NlMsgHdr = unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const NlMsgHdr) };

    if hdr.nlmsg_type != NLMSG_ERROR {
        return None;
    }

    if buf.len() < NlMsgHdr::SIZE + NlMsgErr::SIZE {
        return None;
    }

    let err: NlMsgErr =
        unsafe { core::ptr::read_unaligned(buf[NlMsgHdr::SIZE..].as_ptr() as *const NlMsgErr) };

    Some(err.error)
}

/// returns true if the chunk contains an end message
pub fn recv_is_done(buf: &[u8]) -> bool {
    let mut pos = 0;

    loop {
        if pos + NlMsgHdr::SIZE > buf.len() {
            break;
        }

        let hdr: NlMsgHdr =
            unsafe { core::ptr::read_unaligned(buf[pos..].as_ptr() as *const NlMsgHdr) };

        if hdr.nlmsg_type == NLMSG_DONE || hdr.nlmsg_type == NLMSG_ERROR {
            return true;
        }

        if hdr.nlmsg_flags & NLM_F_MULTI == 0 {
            return true;
        }

        let msg_len = hdr.nlmsg_len as usize;
        if msg_len < NlMsgHdr::SIZE {
            return true;
        }

        pos += nlmsg_align(msg_len);
    }

    false
}
