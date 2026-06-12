# EWOR form — copy/paste helpers

## link

https://github.com/Behruz1306/ewor-bf-chat

## video

*(add when recorded)*  
https://

---

## short answer (form text)

Built a 1:1 TCP chat where the application logic lives in Brainfuck (`server.bf` + `client.bf`). Pure brainfuck can't do networking, so I extended it with BFA: in `--bfa` mode, `.` invokes syscalls via a fixed call frame in cells 0–7. A Rust interpreter handles TCP; a small codegen emits the `.bf` programs.

Repo includes working demo instructions, tests, CI, and write-ups on trade-offs. Hardest bugs: passing cell index instead of fd value, destructive BF copy zeroing the socket fd, and a deadlock when both sides waited on network read first.

---

## slightly longer (if they give more space)

The case study is intentionally impossible in standard brainfuck — that's the point. I treated it like a real constraint: extend the platform minimally (BFA syscalls), keep domain logic in `.bf`, ship something runnable.

Stack: `bf-run --bfa` runtime, `bf-gen` for program emission, blocking line-based chat on localhost:4242. Client types first each round; host replies. Not production-grade async, but genuine two-process 1:1 messaging with logic in brainfuck.

I documented decisions (`DECISIONS.md`), a dev log (`notes.md`), and left a slot for a short screen recording in the README.

---

## if they ask live

Be ready to explain:

1. Why pure BF fails (no I/O beyond stdin/stdout)  
2. What BFA is (`. = syscall`, cells 0–7)  
3. The fd copy bug and preserve-copy fix  
4. Why client types first (deadlock otherwise)  
5. What you'd add with more time (`poll`, proper sockaddr, `.bfl` syntax)
