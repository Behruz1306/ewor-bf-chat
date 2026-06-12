use crate::bfa::{BfaMachine, RunError};

pub fn run(source: &str, bfa: bool) -> Result<(), RunError> {
    if bfa {
        BfaMachine::new(source)?.run()
    } else {
        run_classic(source)
    }
}

fn run_classic(source: &str) -> Result<(), RunError> {
    let mut tape = vec![0u8; 30_000];
    let mut ptr: usize = 0;
    let mut ip: usize = 0;
    let jumps = build_jump_table(source)?;

    while ip < source.len() {
        match source.as_bytes()[ip] {
            b'>' => {
                ptr = ptr.saturating_add(1).min(tape.len() - 1);
            }
            b'<' => {
                ptr = ptr.saturating_sub(1);
            }
            b'+' => {
                tape[ptr] = tape[ptr].wrapping_add(1);
            }
            b'-' => {
                tape[ptr] = tape[ptr].wrapping_sub(1);
            }
            b'.' => {
                print!("{}", tape[ptr] as char);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            b',' => {
                let mut buf = [0u8; 1];
                if std::io::Read::read(&mut std::io::stdin(), &mut buf).unwrap_or(0) == 0 {
                    tape[ptr] = 0;
                } else {
                    tape[ptr] = buf[0];
                }
            }
            b'[' => {
                if tape[ptr] == 0 {
                    ip = jumps[ip];
                }
            }
            b']' => {
                if tape[ptr] != 0 {
                    ip = jumps[ip];
                }
            }
            _ => {}
        }
        ip += 1;
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world() {
        let src = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        run_classic(src).unwrap();
    }
}
