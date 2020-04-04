use super::shutdown;
use super::Agent;
use super::State;
use crate::util;
use mio::*;
use std::io;
use std::net::Shutdown;

pub fn relay_in(agt: &mut Agent, r: &Registry, token: usize) -> io::Result<()> {
    // s1 is Client Connection.
    // s2 is Target Host Connection.
    // b1 is Client Buffer.
    let (s1, s2, b) = if token == agt.token {
        (&mut agt.s1, agt.s2.as_mut().unwrap(), &mut agt.b1)
    } else if let Some(s2) = &mut agt.s2 {
        (s2, &mut agt.s1, &mut agt.b2)
    } else {
        // Exception !!
        unreachable!()
    };

    // Read to Client Buffer from Client Connection.
    // Write to Target Host Connection from Client Buffer.
    // 从客户端的连接中读取数据到客户端缓冲区中，在从缓冲区中读取数据到代理目标连接中.
    // ea is Successful ??
    let ea = b.copy(s1, s2)?;
    if b.len() > 0 || ea {
        return Ok(()); // write EAGAIN || read EAGAIN
    }

    // s1被关闭了

    // 关闭s2的写
    s2.shutdown(Shutdown::Write)?;

    // 取消s1的epollin事件
    r.reregister(&s1, Token(token), Interests::WRITABLE)?;

    // 取消s2的epollout事件
    r.reregister(&s2, Token(util::peer_token(token)), Interests::READABLE)?;

    agt.set_state(State::Shutdown);
    shutdown::shutdown(agt, r)
}

pub fn relay_out(c: &mut Agent, r: &Registry, token: usize) -> io::Result<()> {
    relay_in(c, r, util::peer_token(token))
}
