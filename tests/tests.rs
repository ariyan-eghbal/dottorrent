use libdottorrent;
use std::fs::File;
use std::io::Read;

#[test]
fn loads_torrent_v1_single_file(){
    let path = "tests/samples/debian-12.5.0-amd64-DVD-1.iso.torrent";
    let f = File::open(path);
    assert!(f.is_ok());
    let mut f = f.unwrap();

    let metadata = std::fs::metadata(path);
    assert!(metadata.is_ok());
    let metadata = metadata.unwrap();

    let mut buffer = vec![0; metadata.len() as usize];
    assert!(f.read(&mut buffer).is_ok());
    
    let t = libdottorrent::Torrent::from_bytes(&buffer).unwrap();
    assert_eq!(t.is_single(), true);
    assert_eq!(t.size().unwrap(), 3992977408u128);
    assert_eq!(t.files_count().unwrap(), 1);
}

#[test]
fn loads_torrent_v1_multi_file(){
    let path = "tests/samples/big-buck-bunny.torrent";
    let f = File::open(path);
    assert!(f.is_ok());
    let mut f = f.unwrap();

    let metadata = std::fs::metadata(path);
    assert!(metadata.is_ok());
    let metadata = metadata.unwrap();

    let mut buffer = vec![0; metadata.len() as usize];
    assert!(f.read(&mut buffer).is_ok());
    
    let t = libdottorrent::Torrent::from_bytes(&buffer).unwrap();
    assert_eq!(t.is_single(), false);
    assert_eq!(t.size().unwrap(), 276445467u128);
    assert_eq!(t.files_count().unwrap(), 3);
}

