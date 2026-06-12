# implementation notes

## constraints

Standard brainfuck: 8 ops, stdin/stdout only. Networking requires extending the runtime.

## bugs hit

1. **fd as cell index** — passed `8` instead of the value stored in cell 8  
2. **destructive copy** — fd zeroed after bind; listen failed with `fd=0`  
3. **deadlock** — both sides blocked on network read before stdin  
4. **codegen pointer drift** — simplified constant emission to plain `+` runs  

## chat behaviour

Line-based, blocking. Client sends first each round; host replies. Prompts in the programs mark where to type.

## with more time

- `poll` / `select` syscall for non-blocking I/O  
- portable sockaddr parsing (Linux + macOS)  
- higher-level `.bfl` syntax compiling to BFA  
