use serde_bencode::de;
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use std::fmt::Display;

#[allow(unused)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Node(String, i64);

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    md5sum: Option<String>,
    /// The path to the file within the torrent structure
    path: Vec<String>,
    /// The total size of the file or files in bytes
    length: i64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    /// A list of dictionaries representing individual files within the torrent (if multiple files are shared)
    /// mutual exclusive with `files` filed, one of them is required
    #[serde(default)]
    pub files: Option<Vec<File>>,
    /// size of the file in bytes (only when one file is being shared though)
    /// mutual exclusive with `files` filed, one of them is required
    #[serde(default)]
    pub length: Option<i64>,
    pub md5sum: Option<String>,
    /// Bittorrent file format V2:torrent standard version, in version 1 it is missing
    #[serde(rename(deserialize = "meta version", serialize = "meta version"))]
    pub meta_version: Option<i32>,
    /// suggested name to save the file (or directory if it is multi-file)
    pub name: String,
    /// Bittorrent file format V1: a concatenation of each piece's SHA-1 hash.
    /// This is a multiply of 20
    pub pieces: ByteBuf,
    /// Bittorrent file format V1: The size of each piece in bytes
    #[serde(rename(serialize = "piece length", deserialize = "piece length"))]
    pub piece_length: i64,
    // TODO: file tree
    // TODO: piece layers
    #[serde(default)]
    pub private: Option<u8>,
    //#[serde(default)]
    //#[serde(rename(deserialize = "root hash", serialize = "root hash"))]
    //pub root_hash: Option<String>,
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\t{}", self.name)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Torrent {
    /// Bittorrent file format V1: Tracker address
    // #[serde(default)]
    pub announce: Option<String>,
    /// Bittorrent file format Ext(12): list of lists,
    /// inner list represents tiers and each tier must fail to go to the next.
    /// `announce` will be ignored if client support this
    // #[serde(default)]
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    // #[serde(default)]
    #[serde(rename = "creation date")]
    pub creation_date: Option<i64>,
    #[serde(rename = "comment")]
    pub comment: Option<String>,
    // #[serde(default)]
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
    // #[serde(default)]
    pub encoding: Option<String>,
    /// Bittorrent file format: Info dictionary
    pub info: Info,
    /// Bittorrent file format Ext(5): DHT Nodes
    #[serde(default)]
    pub nodes: Option<Vec<Node>>,
    /// A list of HTTP seed URLs for direct file downloading.
    #[serde(default)]
    pub httpseeds: Option<Vec<String>>,
}

impl Torrent {
    pub fn total_size(&self) -> u128 {
        if let Some(files) = &self.info.files {
            files.iter().fold(0u128, |acc, v| acc + v.length as u128)
        } else if let Some(length) = self.info.length {
            length as u128
        } else {
            0u128
        }
    }

    pub fn files_count(&self) -> usize {
        if let Some(files) = &self.info.files {
            files.len()
        } else if self.info.length.is_some() {
            1
        } else {
            0
        }
    }

    pub fn pieces_count(&self) -> usize {
        self.info.pieces.len() / 20
    }

    pub fn is_single(&self) -> bool {
        self.info.files.is_some()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Torrent, serde_bencode::Error> {
        de::from_bytes::<Torrent>(bytes)
    }

    pub fn sha1(&self) -> String {
        let mut hasher = Sha1::new();
        let bencoded_info = serde_bencode::to_bytes(&self.info).unwrap();
        // let bencoded_info = "Test".as_bytes().to_vec();
        hasher.update(&bencoded_info);
        // hasher.update("Test");
        let hash = hasher.finalize();
        // log!("BEncoded Info: {:?}", &bencoded_info);
        // log!("SHA1: {:?}", hash);
        hex::encode(hash)
    }

    pub fn md5(&self) -> String {
        let mut hasher = md5::Md5::new();
        hasher.update(serde_bencode::to_bytes(&self.info).unwrap());
        let hash = hasher.finalize();
        // log!("MD5: {:?}", hash);
        hex::encode(hash)
    }
}

fn _render_torrent(torrent: &Torrent) {
    println!("name:\t\t{}", torrent.info.name);
    println!("announce:\t{:?}", torrent.announce);
    if let Some(ans_list) = &torrent.announce_list {
        println!("announce list:\n");
        ans_list.iter().for_each(|item| {
            println!("\t-------------------------");
            item.iter().for_each(|ans| println!("\t{}", ans))
        });
    }
    if let Some(seeds) = &torrent.httpseeds {
        println!("Http Seeds:\n");
        seeds.iter().for_each(|item| println!("\t{}", item));
    }

    println!("nodes:\t\t{:?}", torrent.nodes);
    if let Some(al) = &torrent.announce_list {
        for a in al {
            println!("announce list:\t{}", a[0]);
        }
    }
    println!("httpseeds:\t{:?}", torrent.httpseeds);
    println!("creation date:\t{:?}", torrent.creation_date);
    println!("comment:\t{:?}", torrent.comment);
    println!("created by:\t{:?}", torrent.created_by);
    println!("encoding:\t{:?}", torrent.encoding);
    println!("piece length:\t{:?}", torrent.info.piece_length);
    println!("private:\t{:?}", torrent.info.private);
    //println!("root hash:\t{:?}", torrent.info.root_hash);
    println!("md5sum:\t\t{:?}", torrent.info.md5sum);
    //println!("path:\t\t{:?}", torrent.info.path);
    if let Some(files) = &torrent.info.files {
        for f in files {
            println!("file path:\t{:?}", f.path);
            println!("file length:\t{}", f.length);
            println!("file md5sum:\t{:?}", f.md5sum);
        }
    }
}
