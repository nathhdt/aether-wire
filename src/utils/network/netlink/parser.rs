//! Netlink parsing module

use super::constants::{NLA_TYPE_MASK, NLM_F_MULTI, NLMSG_DONE, NLMSG_ERROR};
use super::types::{IfInfoMsg, NlMsgErr, NlMsgHdr, RtAttr};

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

#[inline]
fn read_struct<T: Copy>(buf: &[u8], pos: usize) -> Option<T> {
    let end = pos.checked_add(core::mem::size_of::<T>())?;
    if end > buf.len() {
        return None;
    }

    Some(unsafe { core::ptr::read_unaligned(buf.as_ptr().add(pos) as *const T) })
}

/// reads an ifinfomsg from a RTM_NEWLINK payload
pub fn parse_ifinfomsg(payload: &[u8]) -> Option<(IfInfoMsg, &[u8])> {
    let hdr = read_struct(payload, 0)?;
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
        let hdr: NlMsgHdr = read_struct(self.buf, self.pos)?;

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
        let attr: RtAttr = read_struct(self.buf, self.pos)?;

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
    let hdr: NlMsgHdr = read_struct(buf, 0)?;

    if hdr.nlmsg_type != NLMSG_ERROR {
        return None;
    }

    let err: NlMsgErr = read_struct(buf, NlMsgHdr::SIZE)?;

    if err.error != 0 {
        Some(err.error)
    } else {
        None
    }
}

/// returns true if the chunk contains an end message
pub fn recv_is_done(buf: &[u8]) -> bool {
    let mut pos = 0;

    while let Some(hdr) = read_struct::<NlMsgHdr>(buf, pos) {
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
