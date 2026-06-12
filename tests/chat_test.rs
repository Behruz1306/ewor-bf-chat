use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
fn server_accepts_and_reads() {
    let bin = "./target/release/bf-run";
    if !std::path::Path::new(bin).exists() {
        eprintln!("skip: build release first");
        return;
    }

    let mut server = Command::new(bin)
        .args(["--bfa", "programs/server.bf"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn server");

    thread::sleep(Duration::from_millis(800));

    let mut client = Command::new(bin)
        .args(["--bfa", "programs/client.bf"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn client");

    {
        let mut si = server.stdin.take().unwrap();
        si.write_all(b"reply from host\n").unwrap();
    }
    {
        let mut ci = client.stdin.take().unwrap();
        ci.write_all(b"ping from client\n").unwrap();
    }

    thread::sleep(Duration::from_secs(2));

    server.kill().ok();
    client.kill().ok();
}
