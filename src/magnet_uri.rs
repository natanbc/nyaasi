//https://github.com/pierd/magnet-uri/blob/master/src/lib.rs
//changed BITTORRENT_INFO_HASH to be "urn:btih:" instead of "urn:bith:"

extern crate serde_urlencoded;

use serde_urlencoded::de::Error as UrlEncodeError;
use std::fmt;
use std::str::FromStr;

const SCHEME: &str = "magnet:?";

pub(self) mod field_name {
    pub const NAME: &str = "dn";
    pub const LENGTH: &str = "xl";
    pub const TOPIC: &str = "xt";
    pub const ACCEPTABLE_SOURCE: &str = "as";
    pub const EXACT_SOURCE: &str = "xs";
    pub const KEYWORD: &str = "kt";
    pub const MANIFEST: &str = "mt";
    pub const ADDRESS_TRACKER: &str = "tr";
    pub const EXTENSION_PREFIX: &str = "x.";
}

pub(self) mod exact_topic_urn {
    pub const TIGER_TREE_HASH: &str = "urn:tree:tiger:";
    pub const SHA1: &str = "urn:sha1:";
    pub const BIT_PRINT: &str = "urn:bitprint:";
    pub const ED2K: &str = "urn:ed2k:";
    pub const AICH: &str = "urn:aich:";
    pub const KAZAA: &str = "urn:kzhash:";
    pub const BITTORRENT_INFO_HASH: &str = "urn:btih:";
    pub const MD5: &str = "urn:md5:";
}

#[derive(Debug)]
pub enum Error {
    Scheme,
    UrlEncode(UrlEncodeError),
    Field(String, String),
    ExactTopic(String),
}

impl Error {
    fn with_field(key: &str, val: &str) -> Self {
        Error::Field(key.to_owned(), val.to_owned())
    }
}

/// A struct holding fields stored in a Magnet URI
#[derive(Debug, Default)]
pub struct MagnetURI {
    fields: Vec<Field>,
}

impl MagnetURI {
    pub fn has_extensions(&self) -> bool {
        self.fields.iter().any(Field::is_extension)
    }

    pub fn has_unknown_fields(&self) -> bool {
        self.fields.iter().any(Field::is_unknown)
    }

    pub fn has_topic_conflict(&self) -> bool {
        self.iter_topics().any(|topic1| {
            self.iter_topics()
                .any(|topic2| Topic::conflicts(topic1, topic2))
        })
    }

    pub fn is_strictly_valid(&self) -> bool {
        !self.has_unknown_fields() && self.length() != None && !self.has_topic_conflict()
    }

    pub fn names(&self) -> Vec<&str> {
        self.iter_field_values(Field::name).collect()
    }

    pub fn name(&self) -> Option<&str> {
        self.iter_field_values(Field::name).next()
    }

    pub fn dn(&self) -> Option<&str> {
        self.name()
    }

    pub fn length(&self) -> Option<u64> {
        self.iter_field_values(Field::length).next()
    }

    pub fn xl(&self) -> Option<u64> {
        self.length()
    }

    pub fn iter_topics(&self) -> impl Iterator<Item = &Topic> {
        self.iter_field_values(Field::topic)
    }

    pub fn topics(&self) -> Vec<&Topic> {
        self.iter_topics().collect()
    }

    pub fn info_hashes(&self) -> Vec<&BTInfoHash> {
        self.iter_field_values(Field::info_hash).collect()
    }

    pub fn info_hash(&self) -> Option<&BTInfoHash> {
        self.iter_field_values(Field::info_hash).next()
    }

    fn iter_field_values<'a, F, T>(&'a self, f: F) -> impl Iterator<Item = T> + 'a
    where
        F: Fn(&'a Field) -> Option<T> + Sized + 'a,
        T: 'a,
    {
        self.fields
            .iter()
            .map(f)
            .filter(Option::is_some)
            .map(Option::unwrap)
    }

    pub fn add_field(&mut self, f: Field) -> &Self {
        self.fields.push(f);
        self
    }

    pub fn add_name(&mut self, name: &str) -> &Self {
        self.add_field(Field::Name(name.to_owned()))
    }

    pub fn add_topic(&mut self, xt: Topic) -> &Self {
        self.add_field(Field::Topic(xt))
    }

    pub fn add_extension(&mut self, ext_name: &str, val: &str) -> &Self {
        self.add_field(Field::Extension(ext_name.to_owned(), val.to_owned()))
    }

    pub fn set_name(&mut self, name: &str) -> &Self {
        self.set_unique_field(|f| f.name().is_none(), Field::Name(name.to_owned()))
    }

    pub fn set_info_hash(&mut self, bith: BTInfoHash) -> &Self {
        self.set_unique_field(
            |f| match f {
                Field::Topic(Topic::BitTorrentInfoHash(_)) => false,
                _ => true,
            },
            Field::Topic(Topic::BitTorrentInfoHash(bith)),
        )
    }

    fn set_unique_field<F>(&mut self, retain_filter: F, field: Field) -> &Self
    where
        F: FnMut(&Field) -> bool,
    {
        self.fields.retain(retain_filter);
        self.add_field(field)
    }
}

impl FromStr for MagnetURI {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with(SCHEME) {
            return Err(Error::Scheme);
        }
        let (_, qs) = s.split_at(SCHEME.len());
        let parse_result: Result<Vec<(String, String)>, _> = serde_urlencoded::from_str(qs);
        match parse_result {
            Err(e) => Err(Error::UrlEncode(e)),
            Ok(parts) => Ok(MagnetURI {
                fields: parts
                    .iter()
                    .map(|(k, v)| Field::from_str(k, v))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
        }
    }
}

/// Field of a Magnet URI
#[derive(Debug, PartialEq)]
pub enum Field {
    Name(String),
    Length(u64),
    Topic(Topic),
    AcceptableSource(String),
    ExactSource(String),
    Keyword(String),
    Manifest(String),
    AddressTracker(String),
    Extension(String, String),
    Unknown(String, String),
}

impl Field {
    fn from_str(key: &str, val: &str) -> Result<Self, Error> {
        use field_name::*;
        use Field::*;

        match key {
            NAME => Ok(Name(val.to_owned())),
            LENGTH => match u64::from_str(val) {
                Err(_) => Err(Error::with_field(key, val)),
                Ok(l) => Ok(Length(l)),
            },
            TOPIC => Ok(Topic(self::Topic::from_str(val)?)),
            ACCEPTABLE_SOURCE => Ok(AcceptableSource(val.to_owned())),
            EXACT_SOURCE => Ok(ExactSource(val.to_owned())),
            KEYWORD => Ok(Keyword(val.to_owned())),
            MANIFEST => Ok(Manifest(val.to_owned())),
            ADDRESS_TRACKER => Ok(AddressTracker(val.to_owned())),
            _ => {
                if key.starts_with(EXTENSION_PREFIX) {
                    let (_, ext_name) = key.split_at(EXTENSION_PREFIX.len());
                    Ok(Extension(ext_name.to_owned(), val.to_owned()))
                } else {
                    Ok(Unknown(key.to_owned(), val.to_owned()))
                }
            }
        }
    }

    fn is_extension(&self) -> bool {
        match self {
            Field::Extension(_, _) => true,
            _ => false,
        }
    }

    fn is_unknown(&self) -> bool {
        match self {
            Field::Unknown(_, _) => true,
            _ => false,
        }
    }

    fn name(&self) -> Option<&str> {
        match self {
            Field::Name(ref name) => Some(name),
            _ => None,
        }
    }

    fn length(&self) -> Option<u64> {
        match self {
            Field::Length(len) => Some(*len),
            _ => None,
        }
    }

    fn topic(&self) -> Option<&Topic> {
        match self {
            Field::Topic(topic) => Some(topic),
            _ => None,
        }
    }

    fn info_hash(&self) -> Option<&BTInfoHash> {
        match self {
            Field::Topic(Topic::BitTorrentInfoHash(ref hash)) => Some(hash),
            _ => None,
        }
    }
}

// TODO: use more specific types
type TTHHash = String;
type SHA1Hash = String;
type ED2KHash = String;
type AICHHash = String;
type KazaaHash = String;
type BTInfoHash = String;
type MD5Hash = String;

/// Topic (hash) of a Magnet URI
#[derive(Debug, PartialEq)]
pub enum Topic {
    /// urn:tree:tiger:TTHHash
    TigerTreeHash(TTHHash),
    /// urn:sha1:SHA1Hash
    SHA1(SHA1Hash),
    /// urn:bitprint:SHA1Hash.TTHHash
    BitPrint(SHA1Hash, TTHHash),
    /// urn:ed2k:ED2KHash
    ED2K(ED2KHash),
    /// urn:aich:AICHHash
    AICH(AICHHash),
    /// urn:kzhash:KazaaHash
    Kazaa(KazaaHash),
    /// urn:bith:BTInfoHash
    BitTorrentInfoHash(BTInfoHash),
    /// urn:md5:MD5Hash
    MD5(MD5Hash),
}

impl Topic {
    fn conflicts(&self, other: &Topic) -> bool {
        use Topic::*;

        match (self, other) {
            (TigerTreeHash(h1), TigerTreeHash(h2)) => h1 != h2,
            (SHA1(h1), SHA1(h2)) => h1 != h2,
            (BitPrint(sha1, tth1), BitPrint(sha2, tth2)) => sha1 != sha2 || tth1 != tth2,
            (ED2K(h1), ED2K(h2)) => h1 != h2,
            (AICH(h1), AICH(h2)) => h1 != h2,
            (Kazaa(h1), Kazaa(h2)) => h1 != h2,
            (BitTorrentInfoHash(h1), BitTorrentInfoHash(h2)) => h1 != h2,
            (MD5(h1), MD5(h2)) => h1 != h2,

            (TigerTreeHash(tth1), BitPrint(_, tth2)) => tth1 != tth2,
            (BitPrint(_, tth1), TigerTreeHash(tth2)) => tth1 != tth2,
            (SHA1(sha1), BitPrint(sha2, _)) => sha1 != sha2,
            (BitPrint(sha1, _), SHA1(sha2)) => sha1 != sha2,

            _ => false,
        }
    }
}

impl FromStr for Topic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Topic::*;

        if let Some(hash) = match_prefix(s, exact_topic_urn::TIGER_TREE_HASH) {
            Ok(TigerTreeHash(hash.to_owned()))
        } else if let Some(hash) = match_prefix(s, exact_topic_urn::SHA1) {
            Ok(SHA1(hash.to_owned()))
        } else if let Some(hashes) = match_prefix(s, exact_topic_urn::BIT_PRINT) {
            let mut parts = hashes.split('.');
            if let (Some(sha_hash), Some(tth_hash), None) =
                (parts.next(), parts.next(), parts.next())
            {
                Ok(BitPrint(sha_hash.to_owned(), tth_hash.to_owned()))
            } else {
                Err(Error::ExactTopic(s.to_owned()))
            }
        } else if let Some(hash) = match_prefix(s, exact_topic_urn::ED2K) {
            Ok(ED2K(hash.to_owned()))
        } else if let Some(hash) = match_prefix(s, exact_topic_urn::AICH) {
            Ok(AICH(hash.to_owned()))
        } else if let Some(hash) = match_prefix(s, exact_topic_urn::KAZAA) {
            Ok(Kazaa(hash.to_owned()))
        } else if let Some(hash) = match_prefix(s, exact_topic_urn::BITTORRENT_INFO_HASH) {
            Ok(BitTorrentInfoHash(hash.to_owned()))
        } else if let Some(hash) = match_prefix(s, exact_topic_urn::MD5) {
            Ok(MD5(hash.to_owned()))
        } else {
            Err(Error::ExactTopic(s.to_owned()))
        }
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Topic::*;
        match self {
            TigerTreeHash(hash) => write!(f, "{}{}", exact_topic_urn::TIGER_TREE_HASH, hash),
            SHA1(hash) => write!(f, "{}{}", exact_topic_urn::SHA1, hash),
            BitPrint(hash1, hash2) => {
                write!(f, "{}{}.{}", exact_topic_urn::BIT_PRINT, hash1, hash2)
            }
            ED2K(hash) => write!(f, "{}{}", exact_topic_urn::ED2K, hash),
            AICH(hash) => write!(f, "{}{}", exact_topic_urn::AICH, hash),
            Kazaa(hash) => write!(f, "{}{}", exact_topic_urn::KAZAA, hash),
            BitTorrentInfoHash(hash) => {
                write!(f, "{}{}", exact_topic_urn::BITTORRENT_INFO_HASH, hash)
            }
            MD5(hash) => write!(f, "{}{}", exact_topic_urn::MD5, hash),
        }
    }
}

fn match_prefix<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.starts_with(prefix) {
        let (_, postfix) = s.split_at(prefix.len());
        Some(postfix)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_prefix() {
        assert_eq!(match_prefix("foobar", "foobar"), Some(""));
        assert_eq!(match_prefix("foobar", "foo"), Some("bar"));
        assert_eq!(match_prefix("foobar", "foob"), Some("ar"));
        assert_eq!(match_prefix("foobar", "baz"), None);
    }

    #[test]
    fn test_zero_file_parsing() {
        let uri = MagnetURI::from_str("magnet:?xt=urn:ed2k:31D6CFE0D16AE931B73C59D7E0C089C0&xl=0&dn=zero_len.fil&xt=urn:bitprint:3I42H3S6NNFQ2MSVX7XZKYAYSCX5QBYJ.LWPNACQDBZRYXW3VHJVCJ64QBZNGHOHHHZWCLNQ&xt=urn:md5:D41D8CD98F00B204E9800998ECF8427E").unwrap();
        assert!(uri.is_strictly_valid());
        assert_eq!(uri.length(), Some(0));
        assert_eq!(
            uri.topics(),
            vec![
                &Topic::ED2K("31D6CFE0D16AE931B73C59D7E0C089C0".to_owned()),
                &Topic::BitPrint(
                    "3I42H3S6NNFQ2MSVX7XZKYAYSCX5QBYJ".to_owned(),
                    "LWPNACQDBZRYXW3VHJVCJ64QBZNGHOHHHZWCLNQ".to_owned()
                ),
                &Topic::MD5("D41D8CD98F00B204E9800998ECF8427E".to_owned()),
            ]
        );
    }

    #[test]
    fn test_invalid_non_matching_hashes() {
        let uri = MagnetURI::from_str("magnet:?xt=urn:md5:31D6CFE0D16AE931B73C59D7E0C089C0&xl=0&dn=zero_len.fil&xt=urn:bitprint:3I42H3S6NNFQ2MSVX7XZKYAYSCX5QBYJ.LWPNACQDBZRYXW3VHJVCJ64QBZNGHOHHHZWCLNQ&xt=urn:md5:D41D8CD98F00B204E9800998ECF8427E").unwrap();
        assert!(!uri.is_strictly_valid());
    }

    #[test]
    fn test_invalid_no_length() {
        let uri = MagnetURI::from_str("magnet:?xt=urn:ed2k:31D6CFE0D16AE931B73C59D7E0C089C0&dn=zero_len.fil&xt=urn:bitprint:3I42H3S6NNFQ2MSVX7XZKYAYSCX5QBYJ.LWPNACQDBZRYXW3VHJVCJ64QBZNGHOHHHZWCLNQ&xt=urn:md5:D41D8CD98F00B204E9800998ECF8427E").unwrap();
        assert!(!uri.is_strictly_valid());
        assert_eq!(uri.length(), None);
    }
}
