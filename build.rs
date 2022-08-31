use std::env;
use copy_to_output::copy_to_output;

fn main() {
    println!("cargo:rerun-if-changed=src/mhycrypto/*");

    cc::Build::new()
        .cpp(true)
        .file("src/mhycrypto/memecrypto.cpp")
        .file("src/mhycrypto/metadata.cpp")
        .file("src/mhycrypto/metadatastringdec.cpp")
        .flag_if_supported("-std=c++11")
        .compile("mhycrypto");

    cc::Build::new()
        .file("src/mhycrypto/aes.c")
        .compile("mhycrypto-aes");

    println!("cargo:rerun-if-changed=keys/*");

    copy_to_output("keys", &env::var("PROFILE").unwrap())
        .expect("Could not copy keys");
}
