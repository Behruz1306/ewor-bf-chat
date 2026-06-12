# notes

EWOR asked for a 1:1 chat in brainfuck. ok.

## day 0 — can this even work?

brainfuck has 8 ops. no sockets. so either:
- extend the language (netfuck-style ^ v), or
- hook `.` to syscalls (what I did — BFA)

I went with BFA because it maps cleanly to "real" OS stuff without inventing a whole new alphabet.

## day 1 — interpreter

cells 0–7 reserved for syscalls. everything else is fair game for the program.

first bug: I passed the *cell index* (8) as the fd instead of the *value stored in cell 8*. classic off-by-concept.

second bug: macOS sockaddr layout broke bind. gave up parsing structs and hardcoded 127.0.0.1:4242 in the runtime. still fills in the tape for show.

third bug: brainfuck copy loops zero the source cell. socket fd lived in cell 8, got wiped after bind, listen saw fd=0. fixed with a preserve-copy via scratch cell 18.

fourth bug: codegen pointer tracking drifted after `[` loops. dropped the "smart" multiply loops, just emit `+++…` for small constants.

## day 2 — chat loop

not a fancy async thing. read line from peer → print. read line from stdin → send. blocking, turn-based feel but it's genuinely two processes talking over TCP and the logic is in `.bf`.

## if I had another week

- select/poll syscall so one side doesn't block the other
- proper sockaddr parsing on linux + mac
- compile from a nicer `.bfl` syntax instead of generating straight from rust structs
- fuzz the interpreter

## run checklist for reviewers

```bash
cargo build --release && make gen
# two terminals:
make run-server
make run-client
```
