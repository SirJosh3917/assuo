use serde::de::Error;
use serde::Deserialize;
use toml::Value;

/// Represents an Assuo Patch File. Every Assuo Patch File has a primary source that it is based off of,
/// and a series of patches that it needs to apply to the source.
#[derive(Debug, Deserialize)]
pub struct AssuoFile {
    /// The primary source of this Assuo File. All Assuo modifications are based off of this copy.
    /// All `spot` values correlate directly to the offset (in bytes) of the original file, and patches
    /// will be applied in the order they are listed in, in the method described.
    ///
    /// This enforces the idea that if you want to modify your modifications, you have to create a new base.
    pub source: AssuoSource,

    /// A list of patches to apply. Each patch is applied sequentially, and all `spot` values correlate directly to
    /// the offset (in bytes) of the original file.
    pub patch: Vec<AssuoPatch>,
}

/// Represents some kind of value Assuo knows how to deal with as a source. Each value can be deciphered into
/// a series of bytes, of which Assuo knows how to insert into the original source.
#[derive(Debug)]
pub enum AssuoSource {
    /// A raw amount of bytes. Not recommended to use for performance reasons, but you can if you want to.
    Bytes(Vec<u8>),
    /// Some text. Plain and simple.
    Text(String),
    /// Fetches data at a given URL, and will use the payload to inject it.
    Url(String),
    /// Reads a file on disk at the given path, and will read the file to inject it.
    File(String),
    /// Reads an Assuo patch file from the URL specified, and after applying that Assuo patch file, uses the resultant
    /// data as part of the modification.
    AssuoUrl(String),
    /// Reads an Assuo patch file from disk, and after applying that Assuo patch file, uses the resultant data as part
    /// of the modification.
    AssuoFile(String),
}

/// Represents a single action of patching.
#[derive(Debug, Deserialize)]
pub struct AssuoPatch {
    /// The type of patching to apply.
    ///
    /// - `pre insert`: Inserts the value of the source right before the `spot`.
    /// - `post insert`: Inserts the value of the source right after the `spot`.
    pub modify: AssuoModSpot,
    /// The position (in raw bytes) of the base source to insert the data at.
    pub spot: usize,
    /// The value to insert.
    pub source: AssuoSource,
}

/// Describes the method which to modify the file.
#[derive(Debug)]
pub enum AssuoModSpot {
    /// Inserts the data right before the spot.
    // PreInsert,
    /// Inserts the data right after the spot.
    PostInsert,
    // PreRemove,
    // PostRemove,
}

impl AssuoSource {
    pub fn resolve(self) -> Vec<u8> {
        match self {
            AssuoSource::Bytes(bytes) => bytes,
            AssuoSource::Text(text) => text.into_bytes(),
            _ => panic!("unimplemented route"),
        }
    }
}

// == ugly serialization stuff below ==

impl<'de> Deserialize<'de> for AssuoModSpot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Value::deserialize(deserializer)? {
            Value::String(string) => match string.as_str() {
                // "pre insert" | "pre ins" => Ok(AssuoModSpot::PreInsert),
                "post insert" | "post ins" => Ok(AssuoModSpot::PostInsert),
                // "pre remove" | "pre rm" => Ok(AssuoModSpot::PreRemove),
                // "post remove" | "post rm" => Ok(AssuoModSpot::PostRemove),
                _ => Err(Error::custom("didnt get right modify type")),
            },
            _ => Err(Error::custom("didnt get a string as payload")),
        }
    }
}

impl<'de> Deserialize<'de> for AssuoSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = toml::Value::deserialize(deserializer)?;

        // TODO: this is hideous but it works and it's good enough, so... :yum:
        match value {
            toml::Value::Table(table) => {
                if table.len() != 1 {
                    Err(serde::de::Error::custom("more than 1"))
                } else {
                    let (name, inner) = table.into_iter().nth(0).unwrap();
                    match inner {
                        toml::Value::Array(array) => {
                            if name != "bytes" {
                                Err(serde::de::Error::custom("got array but didnt get bytes"))
                            } else {
                                let mut bytes = Vec::with_capacity(array.len());
                                for element in array {
                                    let byte = match element {
                                        toml::Value::Integer(i) => {
                                            if i >= 0 && i <= 255 {
                                                i as u8
                                            } else {
                                                return Err(serde::de::Error::custom("when converting byte to int, out of bounds [0, 255]"));
                                            }
                                        }
                                        _ => return Err(serde::de::Error::custom(
                                            "when reading bytes array, didnt get nmumber in array",
                                        )),
                                    };
                                    bytes.push(byte);
                                }
                                Ok(AssuoSource::Bytes(bytes))
                            }
                        }
                        toml::Value::String(string) => match name.as_str() {
                            "text" => Ok(AssuoSource::Text(string)),
                            "url" => Ok(AssuoSource::Url(string)),
                            "file" => Ok(AssuoSource::File(string)),
                            "assuo-url" => Ok(AssuoSource::AssuoUrl(string)),
                            "assuo-file" => Ok(AssuoSource::AssuoFile(string)),
                            _ => Err(serde::de::Error::custom(
                                "didnt get key text/url/file/assuo-url/assuo-file",
                            )),
                        },
                        _ => Err(serde::de::Error::custom("invalid value")),
                    }
                }
            }
            _ => Err(serde::de::Error::custom("not table")),
        }
    }
}
