//! This module holds the data structures used when deserializing an Assuo patch file.

use std::io::ErrorKind;

use async_trait::async_trait;
use serde::de::Error;
use serde::Deserialize;
use toml::Value;

/// Tries to deserialize a string to an AssuoFile
///
/// # Example
/// ```
/// use assuo::models::try_parse;
///
/// let source = r#"
/// [source]
/// text = "Hello, World!"
/// "#;
///
/// assert!(try_parse(source).is_ok())
/// ```
pub fn try_parse(payload: &str) -> Result<AssuoFile, toml::de::Error> {
    toml::from_str(payload)
}

/// Represents an Assuo patch file. Every Assuo patch file has a primary source that it is based off of,
/// and a series of patches that it needs to apply to the source.
#[derive(Debug, Deserialize)]
pub struct AssuoFile<S = AssuoSource> {
    /// The primary source of this Assuo File. All Assuo modifications are based off of this copy.
    /// All `spot` values correlate directly to the offset (in bytes) of the original file, and patches
    /// will be applied in the order they are listed in, in the method described.
    ///
    /// This enforces the idea that if you want to modify your modifications, you have to create a new base.
    pub source: S,

    /// A list of patches to apply. Each patch is applied sequentially, and all `spot` values correlate directly to
    /// the offset (in bytes) of the original file.
    // NOTE: this wouldn't be `Option<Vec<T>>` as `Vec<T>` can already be empty, but having the `Option<>` makes
    // serde not complain when no patches are specified. /shrug
    pub patch: Option<Vec<AssuoPatch>>,
}

/// Represents some kind of value Assuo knows how to deal with as a source. Each value can be deciphered into
/// a series of bytes, of which Assuo knows how to insert into the original source.
#[derive(Debug)]
pub enum AssuoSource {
    /// A raw amount of bytes. Not recommended to use for performance reasons, but you can if you want to.
    Bytes(Vec<u8>),
    /// Some text. Plain and simple.
    Text(String),
    /// Reads a file on disk at the given path, and will read the file to inject it.
    File(String),
    /// Fetches data at a given URL, and will use the payload to inject it.
    Url(String),
    /// Reads an Assuo patch file from disk, and after applying that Assuo patch file, uses the resultant data as part
    /// of the modification.
    AssuoFile(String),
    /// Reads an Assuo patch file from the URL specified, and after applying that Assuo patch file, uses the resultant
    /// data as part of the modification.
    AssuoUrl(String),
}

/// Represents a single action of patching.
#[derive(Debug)]
pub enum AssuoPatch<S = AssuoSource> {
    /// Inserts data at a spot. This entails which direction to insert it in, the spot in the original file to start
    /// inserting data at, and the source to resolve for the bytes to insert.
    Insert {
        way: Direction,
        spot: usize,
        source: S,
    },
    /// Removes data at a spot. This entails which direction to remove data in, the spot in the original file to start
    /// removing data at, and the amount of data to remove.
    Remove {
        way: Direction,
        spot: usize,
        count: usize,
    },
}

/// The direction a modification looks in.
#[derive(Debug)]
pub enum Direction {
    /// Before a given spot. For insertions, this would insert data right before the spot. For removals, this would remove
    /// a certain amount of bytes before the spot.
    Pre,
    /// After a given spot. For insertions, this would insert data right after the spot. For removals, this would remove
    /// a certain amount of bytes after the spot.
    Post,
}

// some mildly ugly stuff

/// When one trait needs to perform some kind of computation and resolve into another, this trait can be used.
#[async_trait]
pub trait Resolvable<R> {
    async fn resolve(self) -> std::io::Result<R>;
}

#[async_trait]
impl Resolvable<Vec<u8>> for AssuoSource {
    async fn resolve(self) -> std::io::Result<Vec<u8>> {
        fn err(kind: ErrorKind, reason: &'static str) -> std::io::Error {
            std::io::Error::new(kind, reason)
        }

        // TODO: clean this up
        match self {
            AssuoSource::Bytes(bytes) => Ok(bytes),
            AssuoSource::Text(string) => Ok(string.into_bytes()),
            AssuoSource::File(file_path) => {
                std::fs::read_to_string(file_path).and_then(|string| Ok(string.into_bytes()))
            }
            AssuoSource::Url(url) => {
                let url = match reqwest::Url::parse(&url) {
                    Ok(url) => url,
                    Err(error) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "the url was invalid",
                        ))
                    }
                };

                let response = match reqwest::get(url).await {
                    Ok(response) => response,
                    Err(error) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotConnected,
                            "couldn't GET the url",
                        ))
                    }
                };

                let bytes = match response.bytes().await {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotConnected,
                            "couldn't read bytes from peer",
                        ))
                    }
                };

                Ok(bytes.to_vec())
            }
            AssuoSource::AssuoFile(file_path) => {
                let payload = match std::fs::read_to_string(file_path)
                    .and_then(|string| Ok(string.into_bytes()))
                    .and_then(|bytes| {
                        String::from_utf8(bytes).map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "couldnt parse bytes into string",
                            )
                        })
                    })
                    .and_then(|payload| {
                        try_parse(&payload).map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "couldnt parse asuo config file",
                            )
                        })
                    }) {
                    Ok(payload) => payload,
                    Err(error) => return Err(error),
                };

                crate::patch::do_patch(payload).await
            }
            AssuoSource::AssuoUrl(url) => {
                let url = match reqwest::Url::parse(&url) {
                    Ok(url) => url,
                    Err(error) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "the url was invalid",
                        ))
                    }
                };

                let response = match reqwest::get(url).await {
                    Ok(response) => response,
                    Err(error) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotConnected,
                            "couldn't GET the url",
                        ))
                    }
                };

                let bytes = match response.bytes().await {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotConnected,
                            "couldn't read bytes from peer",
                        ))
                    }
                };

                let bytes = bytes.to_vec();

                let payload = match String::from_utf8(bytes)
                    .map_err(|_| err(ErrorKind::InvalidData, "invalid string"))
                    .and_then(|string| Ok(string.into_bytes()))
                    .and_then(|bytes| {
                        String::from_utf8(bytes).map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "couldnt parse bytes into string",
                            )
                        })
                    })
                    .and_then(|payload| {
                        try_parse(&payload).map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "couldnt parse asuo config file",
                            )
                        })
                    }) {
                    Ok(payload) => payload,
                    Err(error) => return Err(error),
                };

                crate::patch::do_patch(payload).await
            }
        }
    }
}

#[async_trait]
impl Resolvable<AssuoFile<Vec<u8>>> for AssuoFile {
    // impl<S: Send + Resolvable<Vec<u8>>> Resolvable<AssuoFile<Vec<u8>>> for AssuoFile<S> {
    async fn resolve(self) -> std::io::Result<AssuoFile<Vec<u8>>> {
        let resolved_source = self.source.resolve().await?;

        Ok(AssuoFile {
            source: resolved_source,
            patch: self.patch,
        })
    }
}

#[async_trait]
impl Resolvable<AssuoPatch<Vec<u8>>> for AssuoPatch {
    // impl<S: Send + Resolvable<Vec<u8>>> Resolvable<AssuoPatch<Vec<u8>>> for AssuoPatch<S> {
    async fn resolve(self) -> std::io::Result<AssuoPatch<Vec<u8>>> {
        Ok(match self {
            AssuoPatch::Insert { way, spot, source } => {
                let source = source.resolve().await?;
                AssuoPatch::<Vec<u8>>::Insert { way, spot, source }
            }
            AssuoPatch::Remove { way, spot, count } => {
                AssuoPatch::<Vec<u8>>::Remove { way, spot, count }
            }
        })
    }
}

// == ugly serialization stuff below ==
// todo: cleanup

pub trait TomlDeserialize<'de>: Sized {
    fn deserialize_toml<D>(value: Value) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>;
}

impl<'de, S: TomlDeserialize<'de>> Deserialize<'de> for AssuoPatch<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let table = match Value::deserialize(deserializer)? {
            Value::Table(table) => table,
            _ => return Err(Error::custom("didn't get a table as payload")),
        };

        let action = table.get("do");
        let is_insert = if let Some(action) = action {
            let action = match action {
                Value::String(string) => string,
                _ => {
                    return Err(Error::custom(
                        "expected string for action 'do', didn't get that",
                    ))
                }
            };

            // uppercase because docs have it like this,
            // TODO PERF: explore micro-optimization with branch prediction if it should be uppercase or lowercase
            if action.eq_ignore_ascii_case("INSERT") {
                true
            } else if action.eq_ignore_ascii_case("REMOVE") {
                false
            } else {
                return Err(Error::custom(
                    "expected either 'insert' or 'remove' for 'do'",
                ));
            }
        } else {
            return Err(Error::custom("didn't get key 'do' with insert or remove"));
        };

        // both insert and remove need 'way' and 'spot'
        let way = match table.get("way") {
            Some(way) => way,
            None => return Err(Error::custom("didn't get 'way'")),
        };

        let way = match way {
            toml::Value::String(string) => string,
            _ => return Err(Error::custom("didn't get string for way")),
        };

        let way = match way.as_str() {
            "pre" => Direction::Pre,
            "post" => Direction::Post,
            _ => return Err(Error::custom("didn't get 'pre' or 'post' for 'way'")),
        };

        let spot = match table.get("spot") {
            Some(spot) => spot,
            None => return Err(Error::custom("didn't get 'spot'")),
        };

        let spot = match spot {
            toml::Value::Integer(value) => value.clone() as usize,
            _ => return Err(Error::custom("spot wasn't an integer")),
        };

        if is_insert {
            // TODO: don't clone, and just consume the table
            let source = match table.get("source") {
                Some(value) => value,
                None => return Err(Error::custom("expected source to be specified, it wasn't")),
            }
            .clone();

            let source = S::deserialize_toml::<D>(source)?;

            Ok(AssuoPatch::<S>::Insert { way, spot, source })
        } else {
            let count = match table.get("count") {
                Some(value) => value,
                None => return Err(Error::custom("expected count to be specified, it wasn't")),
            };

            let count = match count {
                Value::Integer(count) => count.clone(),
                _ => return Err(Error::custom("expected count to be integer, it wasn't")),
            } as usize;

            Ok(AssuoPatch::<S>::Remove { way, spot, count })
        }
    }
}

impl<'de> Deserialize<'de> for AssuoSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = toml::Value::deserialize(deserializer)?;
        AssuoSource::deserialize_toml::<D>(value)
    }
}

impl<'de> TomlDeserialize<'de> for AssuoSource {
    fn deserialize_toml<D>(value: Value) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
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
                                Err(serde::de::Error::custom("got array but didn't get bytes"))
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
                                            "when reading bytes array, didn't get number in array",
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
                                "didn't get key text/url/file/assuo-url/assuo-file",
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
