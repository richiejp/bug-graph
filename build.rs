// Copyright (C) 2018 Richard Palethorpe <richiejp@f-m.fm>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

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
