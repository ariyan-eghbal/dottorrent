use serde_bencode::de;
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use std::fmt::Display;

#[derive(Debug, Deserialize, Serialize)]
struct Node(String, i64);

#[derive(Debug)]
pub enum Error {
    InvalidMetaData
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub md5sum: Option<String>,
    /// The path to the file within the torrent structure
    pub path: Vec<String>,
    /// The total size of the file or files in bytes
    pub length: i64,
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
    #[serde(rename = "comment")]
    pub comment: Option<String>,
    // #[serde(default)]
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
    // #[serde(default)]
    #[serde(rename = "creation date")]
    pub creation_date: Option<i64>,
    // #[serde(default)]
    pub encoding: Option<String>,
    /// Bittorrent file format: Info dictionary
    pub info: Info,
    // / Bittorrent file format Ext(5): DHT Nodes
    //#[serde(default)]
    //pub nodes: Option<Vec<Node>>,
    // / A list of HTTP seed URLs for direct file downloading.
    //#[serde(default)]
    //pub httpseeds: Option<Vec<String>>,
}

impl Torrent {
    pub fn size(&self) -> Result<u128, Error> {
        // Single file 
        if let Some(length) = self.info.length {
            Ok(length as u128)
        // Multi file 
        } else if let Some(files) = &self.info.files {
            Ok(files.iter().fold(0u128, |acc, v| acc + v.length as u128))
        } else {
            Err(Error::InvalidMetaData)
        }
    }

    pub fn files_count(&self) -> Result<usize, Error> {
        if self.info.length.is_some() {
            Ok(1)
        } else if let Some(files) = &self.info.files {
            Ok(files.len())
        } else {
            Err(Error::InvalidMetaData)
        }
    }

    pub fn pieces_count(&self) -> usize {
        self.info.pieces.len() / 20
    }
    
    pub fn pieces_hashes(&self) -> Result<Vec<&[u8]>, Error> {
        if self.info.pieces.len() % 20 != 0 {
            Err(Error::InvalidMetaData)
        }else{
            let chunks: Vec<&[u8]> = self.info.pieces.chunks(20).collect();
            Ok(chunks)
        }
    }

    pub fn is_single(&self) -> bool {
        self.info.length.is_some()
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

