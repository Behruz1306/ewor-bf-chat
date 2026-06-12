# design decisions

## BFA instead of new operators

NetFuck adds `^` / `v` for send/receive. I kept standard brainfuck symbols and overloaded `.` in `--bfa` mode to mean syscall. Plain brainfuck still runs without the flag.

## Rust runtime, logic in `.bf`

The chat loop is in generated brainfuck. Rust interprets instructions and handles TCP — not the application logic.

## Codegen

Socket setup in raw brainfuck would be enormous. `bf-gen` emits `server.bf` / `client.bf` from Rust; the committed files are valid brainfuck.

## Hardcoded bind address

Sockaddr bytes are placed on the tape for realism. The runtime binds `127.0.0.1:4242` directly after macOS `sockaddr_in` layout issues in v1.

## Client types first

Both sides initially blocked on network read → deadlock. Host waits for client input first; client reads stdin first. Turn-based, but stable without `poll`.

## Preserve-copy for file descriptors

Standard brainfuck copy loops zero the source cell. The socket fd in cell 8 was destroyed after bind; listen then saw `fd=0`. Fixed with copy-through scratch cell 18.

## Out of scope

TLS, WebSocket, async multiplexing, full `.bfl` language — noted in `notes.md` as future work.
