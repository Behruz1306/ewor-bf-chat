# decisions

Short log of choices I made for the EWOR case study. Not everything was optimal — but each call had a reason.

## 1. BFA instead of NetFuck-style `^` / `v`

NetFuck adds send/receive operators. I reused `.` as syscall trigger in a separate `--bfa` mode so plain `.bf` files still run as normal brainfuck if you drop the flag.

Syscall numbers live in cell 7, args in 1–6, return in 0. Felt closer to "real" systems programming than inventing four new punctuation chars.

## 2. Rust runtime, logic in `.bf`

Could've written the whole chat in Rust and called it a day. Misses the point.

The chat loop (read → write → read → write) is in generated brainfuck. Rust only interprets instructions and talks to the OS. That's the split I'd use in a real project too: thin runtime, weird domain logic isolated.

## 3. Codegen instead of hand-written socket `.bf`

I tried sketching bind/listen in raw BF. Life's too short. `bf-gen` emits the tape setup and syscall sequences from Rust. The committed `.bf` files are the artifact reviewers can open — still 100% brainfuck symbols.

## 4. Hardcoded `127.0.0.1:4242` in the runtime

Sockaddr bytes are still written to the tape (cells 100+) so the program *looks* like it's passing an address. Parsing worked on paper; macOS `sockaddr_in` layout (`sin_len` etc.) broke bind early on.

Pragmatic call: hardcode host in `bfa.rs`, ship a working demo, document the shortcut. Would fix with proper struct parsing if this were production.

## 5. Asymmetric chat loop (client types first)

Both sides started by waiting on network read → instant deadlock.

Fix: host waits for client message first; client reads stdin first. Turn-based, slightly awkward UX, but no deadlocks without `select()`. Prompts (`[client] type here>`) added after I watched myself type in the wrong terminal twice.

## 6. Preserve-copy for file descriptors

Brainfuck copy loops destroy the source cell. Socket fd lived in cell 8; after bind's copy-to-arg, fd was zero → listen got `fd=0`.

Scratch cell 18 + copy-with-restore. Took longer to find than I'd like to admit.

## 7. What I deliberately did NOT build

- TLS, WebSocket, React UI — scope creep for a case study  
- Full `.bfl` language — codegen structs were enough  
- Multiplexed I/O — would need `poll` syscall; noted in notes.md for "another week"
