# bf-chat

EWOR builder case study — 1:1 chat in Brainfuck.

Pure brainfuck can't open TCP sockets. So I added **BFA** (brainfuck + syscalls): in `--bfa` mode the `.` instruction triggers a syscall instead of printing a character. Cells 0–7 are the call frame (number in cell 7, args in 1–6, return in 0).

The `.bf` chat programs are generated from a tiny Rust codegen (`bf-gen`) because hand-writing socket setup in brainfuck would be… a lot of `+` signs.

## quick start

```bash
cargo build --release
make gen        # writes programs/server.bf + programs/client.bf
```

**Terminal 1 — host**

```bash
make run-server
```

**Terminal 2 — client**

```bash
make run-client
```

Type a line, press enter. Host sees it, type a reply on the host side, client sees it. Line-based, blocking, good enough for a demo.

Port is hardcoded to **4242** on `127.0.0.1`.

## layout

```
programs/server.bf   # host: listen, accept one peer, relay lines
programs/client.bf   # client: connect, relay lines
src/bfa.rs           # interpreter + TCP syscalls
src/codegen.rs       # emits the .bf from a higher-level plan
src/bf.rs            # classic brainfuck mode (no syscalls)
```

## syscalls (BFA)

| id | name   | args (cells 1–3)        |
|----|--------|-------------------------|
| 1  | read   | fd, buf, max_len        |
| 2  | write  | fd, buf, len            |
| 3  | close  | fd                      |
| 10 | socket | —                       |
| 11 | bind   | fd, addr*, len          |
| 12 | listen | fd, backlog             |
| 13 | accept | fd                      |
| 14 | connect| fd, addr*, len           |

\*addr is laid out in the tape for realism; bind/connect currently use `127.0.0.1:4242` in the runtime.

Set `BF_TRACE=1` to log syscalls to stderr.

## tests

```bash
make test
```

## notes

See `notes.md` for the messy thought process.
