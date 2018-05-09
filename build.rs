use std::process::{exit, Command};
use std::fs;
use std::io::ErrorKind;

macro_rules! copy_files {
    ($from:expr, $to:expr, $( $file:expr ),+) => (
        $(
            fs::copy(concat!($from, "/", $file), concat!($to, "/", $file))
                .expect(concat!("Copy file: ", $from, "/", $file));
            println!(concat!("Output file ", $to, "/", $file));
        )+
    )
}

fn main() {
    if !Command::new("cargo-web")
        .current_dir("client")
        .arg("deploy")
        .status().expect("Calling cargo-web")
        .success()
    {
        eprintln!("Cargo-web failed to build WASM client!");
        exit(1);
    }

    let canon = fs::canonicalize("res/static")
        .expect("Canonicalize static folder path");
    if let Err(e) = fs::create_dir(canon.clone()) {
        if let ErrorKind::AlreadyExists = e.kind() {
            println!("Static folder already exists: {}", canon.display());
        } else {
            panic!("Can't create static folder ({}): {}", canon.display(), e);
        }
    } else {
        println!("Created static folder: {}", canon.display());
    }

    copy_files!("client/target/deploy", "res/static",
                "index.html",
                "bug-graph-client.js",
                "bug-graph-client.wasm",
                "bulma.min.css");
}
