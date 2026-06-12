use crate::bfa::{
    SYS_ACCEPT, SYS_BIND, SYS_CONNECT, SYS_LISTEN, SYS_READ, SYS_SOCKET, SYS_WRITE,
};

pub const PORT: u16 = 4242;
pub const SOCKADDR: usize = 100;
pub const BUF_NET: usize = 20;
pub const BUF_IN: usize = 55;
pub const SOCK_FD: usize = 8;
pub const PEER_FD: usize = 9;
pub const NREAD: usize = 16;
pub const LOOP: usize = 17;
pub const SCRATCH: usize = 18;
pub const LINE_MAX: u8 = 31;

pub struct Emitter {
    out: String,
    pos: usize,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            out: String::new(),
            pos: 0,
        }
    }

    pub fn finish(self) -> String {
        self.out
    }

    fn raw(&mut self, s: &str) {
        self.out.push_str(s);
    }

    fn goto(&mut self, target: usize) {
        self.seek(target);
    }

    fn clear_cell(&mut self, cell: usize) {
        self.seek(cell);
        self.raw("[-]");
    }

    fn set_byte(&mut self, cell: usize, value: u8) {
        self.seek(cell);
        self.raw("[-]");
        if value == 0 {
            return;
        }
        self.seek(cell);
        self.raw(&"+".repeat(value as usize));
    }

    /// move data pointer — only `>`/`<`, no loops, so pos stays accurate
    fn seek(&mut self, target: usize) {
        if target > self.pos {
            self.raw(&">".repeat(target - self.pos));
        } else if target < self.pos {
            self.raw(&"<".repeat(self.pos - target));
        }
        self.pos = target;
    }

    fn copy_to(&mut self, from: usize, to: usize) {
        self.clear_cell(to);
        self.seek(from);
        self.raw("[");
        self.seek(from);
        self.raw("-");
        self.seek(to);
        self.raw("+");
        self.seek(from);
        self.raw("]");
        self.seek(to);
    }

    /// copy `from` -> `to` and restore `from` via SCRATCH
    fn copy_preserve(&mut self, from: usize, to: usize) {
        self.clear_cell(to);
        self.clear_cell(SCRATCH);
        self.seek(from);
        self.raw("[");
        self.seek(from);
        self.raw("-");
        self.seek(to);
        self.raw("+");
        self.seek(SCRATCH);
        self.raw("+");
        self.seek(from);
        self.raw("]");
        self.seek(SCRATCH);
        self.raw("[");
        self.seek(SCRATCH);
        self.raw("-");
        self.seek(from);
        self.raw("+");
        self.seek(SCRATCH);
        self.raw("]");
        self.seek(to);
    }

    fn syscall(&mut self, nr: u8, a1: u8, a2: u8, a3: u8) {
        self.set_byte(7, nr);
        self.set_byte(1, a1);
        self.set_byte(2, a2);
        self.set_byte(3, a3);
        self.set_byte(4, 0);
        self.set_byte(5, 0);
        self.set_byte(6, 0);
        self.goto(7);
        self.raw(".");
    }

    fn syscall_fd(&mut self, nr: u8, fd_cell: usize, a2: u8, a3: u8) {
        self.set_byte(7, nr);
        self.copy_preserve(fd_cell, 1);
        self.set_byte(2, a2);
        self.set_byte(3, a3);
        self.set_byte(4, 0);
        self.set_byte(5, 0);
        self.set_byte(6, 0);
        self.seek(7);
        self.raw(".");
    }

    fn store_result(&mut self, cell: usize) {
        self.copy_to(0, cell);
    }

    fn write_str_at(&mut self, base: usize, text: &str) {
        for (i, b) in text.bytes().enumerate() {
            self.set_byte(base + i, b);
        }
    }

    fn write_stdout(&mut self, base: usize, len: u8) {
        self.syscall(SYS_WRITE, 1, base as u8, len);
    }

    fn say(&mut self, text: &str) {
        self.write_str_at(BUF_NET, text);
        self.write_stdout(BUF_NET, text.len() as u8);
    }

    fn syscall_write_dynamic(&mut self, fd: u8, buf: u8, len_from: usize) {
        self.set_byte(7, SYS_WRITE);
        self.set_byte(1, fd);
        self.set_byte(2, buf);
        self.copy_to(len_from, 3);
        self.set_byte(4, 0);
        self.set_byte(5, 0);
        self.set_byte(6, 0);
        self.goto(7);
        self.raw(".");
    }

    fn syscall_write_fd(&mut self, fd_cell: usize, buf: u8, len_from: usize) {
        self.set_byte(7, SYS_WRITE);
        self.copy_preserve(fd_cell, 1);
        self.set_byte(2, buf);
        self.copy_to(len_from, 3);
        self.set_byte(4, 0);
        self.set_byte(5, 0);
        self.set_byte(6, 0);
        self.seek(7);
        self.raw(".");
    }

    fn setup_sockaddr(&mut self) {
        // macOS/BSD sockaddr_in starts with sin_len; Linux uses AF_INET at offset 0.
        // port + ip land at the same offsets in both layouts (2 and 4).
        #[cfg(target_os = "macos")]
        {
            self.set_byte(SOCKADDR, 16);
            self.set_byte(SOCKADDR + 1, 2);
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.set_byte(SOCKADDR, 2);
            self.set_byte(SOCKADDR + 1, 0);
        }
        let port = PORT.to_be_bytes();
        self.set_byte(SOCKADDR + 2, port[0]);
        self.set_byte(SOCKADDR + 3, port[1]);
        self.set_byte(SOCKADDR + 4, 127);
        self.set_byte(SOCKADDR + 5, 0);
        self.set_byte(SOCKADDR + 6, 0);
        self.set_byte(SOCKADDR + 7, 1);
    }

    fn chat_loop_host(&mut self, peer_cell: usize) {
        self.say("[host] waiting for client...\n");
        self.syscall_fd(SYS_READ, peer_cell, BUF_NET as u8, LINE_MAX);
        self.store_result(NREAD);
        self.goto(NREAD);
        self.raw("[");
        self.syscall_write_dynamic(1, BUF_NET as u8, NREAD);
        self.clear_cell(NREAD);
        self.goto(NREAD);
        self.raw("]");

        self.say("[host] your reply> ");
        self.syscall(SYS_READ, 0, BUF_IN as u8, LINE_MAX);
        self.store_result(NREAD);
        self.goto(NREAD);
        self.raw("[");
        self.syscall_write_fd(peer_cell, BUF_IN as u8, NREAD);
        self.clear_cell(NREAD);
        self.goto(NREAD);
        self.raw("]");
    }

    fn chat_loop_client(&mut self, peer_cell: usize) {
        self.say("[client] type here> ");
        self.syscall(SYS_READ, 0, BUF_IN as u8, LINE_MAX);
        self.store_result(NREAD);
        self.goto(NREAD);
        self.raw("[");
        self.syscall_write_fd(peer_cell, BUF_IN as u8, NREAD);
        self.clear_cell(NREAD);
        self.goto(NREAD);
        self.raw("]");

        self.say("[client] waiting for host...\n");
        self.syscall_fd(SYS_READ, peer_cell, BUF_NET as u8, LINE_MAX);
        self.store_result(NREAD);
        self.goto(NREAD);
        self.raw("[");
        self.syscall_write_dynamic(1, BUF_NET as u8, NREAD);
        self.clear_cell(NREAD);
        self.goto(NREAD);
        self.raw("]");
    }

    fn infinite_loop_start(&mut self) {
        self.set_byte(LOOP, 1);
        self.goto(LOOP);
        self.raw("[");
    }

    fn infinite_loop_end(&mut self) {
        self.goto(LOOP);
        self.raw("]");
    }
}

pub fn generate_server() -> String {
    let mut e = Emitter::new();
    e.setup_sockaddr();

    e.syscall(SYS_SOCKET, 0, 0, 0);
    e.store_result(SOCK_FD);

    e.syscall_fd(SYS_BIND, SOCK_FD, SOCKADDR as u8, 16);
    e.syscall_fd(SYS_LISTEN, SOCK_FD, 1, 0);

    e.write_str_at(BUF_NET, "host listening :4242\n");
    e.write_stdout(BUF_NET, 22);

    e.syscall_fd(SYS_ACCEPT, SOCK_FD, 0, 0);
    e.store_result(PEER_FD);

    e.write_str_at(BUF_NET, "peer connected — wait, client types first\n");
    e.write_stdout(BUF_NET, 44);

    e.infinite_loop_start();
    e.chat_loop_host(PEER_FD);
    e.infinite_loop_end();

    e.finish()
}

pub fn generate_client() -> String {
    let mut e = Emitter::new();
    e.setup_sockaddr();

    e.syscall(SYS_SOCKET, 0, 0, 0);
    e.store_result(SOCK_FD);

    e.syscall_fd(SYS_CONNECT, SOCK_FD, SOCKADDR as u8, 16);

    e.write_str_at(BUF_NET, "connected to host\n");
    e.write_stdout(BUF_NET, 18);

    e.infinite_loop_start();
    e.chat_loop_client(SOCK_FD);
    e.infinite_loop_end();

    e.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_brainfuck_symbols() {
        for src in [generate_server(), generate_client()] {
            for ch in src.chars() {
                assert!("<>+-.,[]".contains(ch), "bad char {ch:?}");
            }
        }
    }
}
