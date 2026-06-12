use bf_chat::codegen;
use std::fs;
use std::path::Path;

fn main() {
    let out = Path::new("programs");
    fs::create_dir_all(out).expect("mkdir programs");

    let server = codegen::generate_server();
    let client = codegen::generate_client();

    fs::write(out.join("server.bf"), &server).expect("write server.bf");
    fs::write(out.join("client.bf"), &client).expect("write client.bf");

    eprintln!(
        "wrote programs/server.bf ({} bytes) and programs/client.bf ({} bytes)",
        server.len(),
        client.len()
    );
}
