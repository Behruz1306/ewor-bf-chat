use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub const SYS_READ: u8 = 1;
pub const SYS_WRITE: u8 = 2;
pub const SYS_CLOSE: u8 = 3;
pub const SYS_SOCKET: u8 = 10;
pub const SYS_BIND: u8 = 11;
pub const SYS_LISTEN: u8 = 12;
pub const SYS_ACCEPT: u8 = 13;
pub const SYS_CONNECT: u8 = 14;

const HOST: &str = "127.0.0.1:4242";

const TAPE_SIZE: usize = 30_000;

#[derive(Debug)]
pub enum RunError {
    UnmatchedBracket(usize),
    Syscall(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for RunError {
    fn from(e: std::io::Error) -> Self {
        RunError::Io(e)
    }
}

enum FdKind {
    Listener(TcpListener),
    Stream(TcpStream),
}

pub struct BfaMachine {
    source: String,
    tape: Vec<u8>,
    ptr: usize,
    ip: usize,
    jumps: Vec<usize>,
    fds: HashMap<i32, FdKind>,
    next_fd: i32,
}

impl BfaMachine {
    pub fn new(source: &str) -> Result<Self, RunError> {
        let jumps = build_jump_table(source)?;
        Ok(Self {
            source: source.to_string(),
            tape: vec![0u8; TAPE_SIZE],
            ptr: 0,
            ip: 0,
            jumps,
            fds: HashMap::new(),
            next_fd: 3,
        })
    }

    pub fn run(mut self) -> Result<(), RunError> {
        while self.ip < self.source.len() {
            match self.source.as_bytes()[self.ip] {
                b'>' => self.ptr = self.ptr.saturating_add(1).min(TAPE_SIZE - 1),
                b'<' => self.ptr = self.ptr.saturating_sub(1),
                b'+' => self.tape[self.ptr] = self.tape[self.ptr].wrapping_add(1),
                b'-' => self.tape[self.ptr] = self.tape[self.ptr].wrapping_sub(1),
                b'.' => self.do_syscall()?,
                b',' => {
                    let mut buf = [0u8; 1];
                    if Read::read(&mut std::io::stdin(), &mut buf).unwrap_or(0) == 0 {
                        self.tape[self.ptr] = 0;
                    } else {
                        self.tape[self.ptr] = buf[0];
                    }
                }
                b'[' => {
                    if self.tape[self.ptr] == 0 {
                        self.ip = self.jumps[self.ip];
                    }
                }
                b']' => {
                    if self.tape[self.ptr] != 0 {
                        self.ip = self.jumps[self.ip];
                    }
                }
                _ => {}
            }
            self.ip += 1;
        }
        Ok(())
    }

    fn arg(&self, idx: usize) -> i32 {
        self.tape.get(idx).copied().unwrap_or(0) as i32
    }

    fn set_ret(&mut self, val: i32) {
        self.tape[0] = val.rem_euclid(256) as u8;
    }

    fn do_syscall(&mut self) -> Result<(), RunError> {
        let nr = self.tape[7];
        let a1 = self.arg(1);
        let a2 = self.arg(2);
        let a3 = self.arg(3);

        if std::env::var("BF_TRACE").is_ok() {
            eprintln!("syscall nr={nr} a1={a1} a2={a2} a3={a3}");
        }

        let ret = match nr {
            SYS_READ => self.sys_read(a1, a2 as usize, a3 as usize)?,
            SYS_WRITE => self.sys_write(a1, a2 as usize, a3 as usize)?,
            SYS_CLOSE => self.sys_close(a1)?,
            SYS_SOCKET => self.sys_socket()?,
            SYS_BIND => self.sys_bind(a1, a2 as usize, a3 as usize)?,
            SYS_LISTEN => self.sys_listen(a1, a2 as usize)?,
            SYS_ACCEPT => self.sys_accept(a1)?,
            SYS_CONNECT => self.sys_connect(a1, a2 as usize, a3 as usize)?,
            _ => return Err(RunError::Syscall(format!("unknown syscall {nr}"))),
        };

        self.set_ret(ret);
        Ok(())
    }

    fn alloc_fd(&mut self, kind: FdKind) -> i32 {
        let fd = self.next_fd;
        self.next_fd += 1;
        self.fds.insert(fd, kind);
        fd
    }

    fn sys_socket(&mut self) -> Result<i32, RunError> {
        let fd = self.next_fd;
        self.next_fd += 1;
        Ok(fd)
    }

    fn sys_bind(&mut self, fd: i32, _ptr: usize, _len: usize) -> Result<i32, RunError> {
        let listener = TcpListener::bind(HOST)?;
        listener.set_nonblocking(true)?;
        self.fds.insert(fd, FdKind::Listener(listener));
        Ok(0)
    }

    fn sys_listen(&mut self, fd: i32, backlog: usize) -> Result<i32, RunError> {
        let _ = backlog;
        match self.fds.get(&fd) {
            Some(FdKind::Listener(_)) => Ok(0),
            _ => Err(RunError::Syscall(format!("listen: bad fd {fd}"))),
        }
    }

    fn sys_accept(&mut self, fd: i32) -> Result<i32, RunError> {
        let listener = match self.fds.get(&fd) {
            Some(FdKind::Listener(l)) => l,
            _ => return Err(RunError::Syscall(format!("accept: bad fd {fd}"))),
        };

        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_read_timeout(Some(Duration::from_millis(100))).ok();
                    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
                    return Ok(self.alloc_fd(FdKind::Stream(stream)));
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(RunError::Io(e)),
            }
        }
    }

    fn sys_connect(&mut self, fd: i32, _ptr: usize, _len: usize) -> Result<i32, RunError> {
        let stream = TcpStream::connect(HOST)?;
        stream.set_read_timeout(Some(Duration::from_millis(100))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
        self.fds.insert(fd, FdKind::Stream(stream));
        Ok(0)
    }

    fn sys_read(&mut self, fd: i32, ptr: usize, len: usize) -> Result<i32, RunError> {
        if fd == 0 {
            let mut buf = vec![0u8; len];
            let n = Read::read(&mut std::io::stdin(), &mut buf)?;
            for (i, b) in buf.iter().take(n).enumerate() {
                if ptr + i < TAPE_SIZE {
                    self.tape[ptr + i] = *b;
                }
            }
            return Ok(n as i32);
        }

        let stream = match self.fds.get_mut(&fd) {
            Some(FdKind::Stream(s)) => s,
            _ => return Err(RunError::Syscall(format!("read: bad fd {fd}"))),
        };

        let mut buf = vec![0u8; len];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => return Ok(0),
                Ok(n) => {
                    for (i, b) in buf.iter().take(n).enumerate() {
                        if ptr + i < TAPE_SIZE {
                            self.tape[ptr + i] = *b;
                        }
                    }
                    return Ok(n as i32);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(RunError::Io(e)),
            }
        }
    }

    fn sys_write(&mut self, fd: i32, ptr: usize, len: usize) -> Result<i32, RunError> {
        if fd == 1 {
            let slice = &self.tape[ptr..ptr.saturating_add(len).min(TAPE_SIZE)];
            Write::write_all(&mut std::io::stdout(), slice)?;
            Write::flush(&mut std::io::stdout())?;
            return Ok(len as i32);
        }

        let stream = match self.fds.get_mut(&fd) {
            Some(FdKind::Stream(s)) => s,
            _ => return Err(RunError::Syscall(format!("write: bad fd {fd}"))),
        };

        let slice = &self.tape[ptr..ptr.saturating_add(len).min(TAPE_SIZE)];
        stream.write_all(slice)?;
        Ok(len as i32)
    }

    fn sys_close(&mut self, fd: i32) -> Result<i32, RunError> {
        self.fds.remove(&fd);
        Ok(0)
    }
}

fn build_jump_table(source: &str) -> Result<Vec<usize>, RunError> {
    let mut stack = Vec::new();
    let mut jumps = vec![0; source.len()];

    for (i, ch) in source.bytes().enumerate() {
        match ch {
            b'[' => stack.push(i),
            b']' => {
                let start = stack.pop().ok_or(RunError::UnmatchedBracket(i))?;
                jumps[start] = i;
                jumps[i] = start;
            }
            _ => {}
        }
    }

    if !stack.is_empty() {
        return Err(RunError::UnmatchedBracket(stack[0]));
    }

    Ok(jumps)
}
