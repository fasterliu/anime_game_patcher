use std::env;

mod metadata_patcher;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("Usage: {} /path/to/global-metadata.dat", &args[0]);
        return;
    }
    println!("The Anime Game metadata patcher");
    println!("Patch: {}", crate::metadata_patcher::patch_metadata(&args[1]));
    return;
}
