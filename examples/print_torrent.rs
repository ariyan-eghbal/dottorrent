use clap::Parser;
use libdottorrent::Torrent;
use std::fs::File;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Torrent file paths
    #[arg(required=true,value_name="path",num_args=1..)]
    paths: Vec<String>,
}

fn main() {
    let args = Args::parse();
    for path in args.paths {
        println!("File: {}", path);
        let mut f = match File::open(&path) {
            Ok(f) => f,
            Err(_) => {
                println!("Error opening file");
                continue;
            }
        };
        let metadata = std::fs::metadata(&path).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        let _ = f.read(&mut buffer).expect("buffer overflow");

        match Torrent::from_bytes(&buffer) {
            Ok(t) => {
                println!("\tName: {}", t.info.name);
                println!("\tNumber of files: {}", t.files_count());
                let size = t.total_size();
                println!(
                    "\tSize: {} B ({})",
                    size,
                    human_bytes::human_bytes(size as f64).as_str()
                );
                println!("\tMD5: {}", t.md5());
                println!("\tSHA1: {}", t.sha1());
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
