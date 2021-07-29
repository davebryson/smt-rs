use anyhow::{anyhow, ensure, Result};
use blake2::{Blake2s, Digest};

pub const LEAF_TAG: u8 = 0;
pub const INTERNAL_TAG: u8 = 1;

pub type EncodedNode = [u8; 65];

/// Used to mark a value for deletion for a given key
pub const DEFAULT_VALUE: &[u8] = b"";

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct HashValue {
    hash: [u8; Self::LENGTH],
}

impl HashValue {
    pub const LENGTH: usize = 32;
    pub const DEPTH: usize = Self::LENGTH * 8;

    pub fn new(data: [u8; Self::LENGTH]) -> Self {
        Self { hash: data }
    }

    /// Create a new HashValue by hashing the `data`
    pub fn digest_of(data: &[u8]) -> Self {
        let mut hash = [0u8; Self::LENGTH];
        let mut hasher = Blake2s::new();
        hasher.update(data);
        hash.copy_from_slice(hasher.finalize().as_ref());
        Self { hash }
    }

    pub fn has_bit_set(&self, index: usize) -> bool {
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self.hash[pos] >> bit) & 1 != 0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.hash.to_vec()
    }

    pub fn placeholder() -> Self {
        Self {
            hash: [0u8; Self::LENGTH],
        }
    }

    pub fn is_placeholder(&self) -> bool {
        self.hash == [0u8; Self::LENGTH]
    }

    pub fn iter_bits(&self) -> HashValueBitIterator<'_> {
        HashValueBitIterator::new(self)
    }

    pub fn common_prefix_bits_len(&self, other: HashValue) -> usize {
        self.iter_bits()
            .zip(other.iter_bits())
            .take_while(|(x, y)| x == y)
            .count()
    }
}

impl AsRef<[u8; HashValue::LENGTH]> for HashValue {
    fn as_ref(&self) -> &[u8; HashValue::LENGTH] {
        &self.hash
    }
}

impl std::ops::Index<usize> for HashValue {
    type Output = u8;

    fn index(&self, s: usize) -> &u8 {
        self.hash.index(s)
    }
}

impl std::fmt::Binary for HashValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.hash {
            write!(f, "{:08b}", byte)?;
        }
        Ok(())
    }
}

impl std::fmt::LowerHex for HashValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.hash {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// An iterator over `HashValue` that generates one bit for each iteration.
pub struct HashValueBitIterator<'a> {
    hash_bytes: &'a [u8],
    pos: std::ops::Range<usize>,
}

impl<'a> HashValueBitIterator<'a> {
    /// Constructs a new `HashValueBitIterator` using given `HashValue`.
    fn new(hash_value: &'a HashValue) -> Self {
        HashValueBitIterator {
            hash_bytes: hash_value.as_ref(),
            pos: (0..HashValue::DEPTH),
        }
    }

    /// Returns the `index`-th bit in the bytes.
    fn get_bit(&self, index: usize) -> bool {
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self.hash_bytes[pos] >> bit) & 1 != 0
    }
}

impl<'a> std::iter::Iterator for HashValueBitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|x| self.get_bit(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pos.size_hint()
    }
}

pub enum Node {
    Internal((HashValue, HashValue)),
    Leaf((HashValue, HashValue)),
}

impl Node {
    pub fn encode(&self) -> Result<(HashValue, EncodedNode)> {
        let mut raw = vec![];
        let mut bits = [0u8; 65];
        match self {
            Node::Leaf((k, v)) => {
                raw.push(LEAF_TAG);
                raw.extend(k.as_ref());
                raw.extend(v.as_ref());
            }
            Node::Internal((l, r)) => {
                raw.push(INTERNAL_TAG);
                raw.extend(l.as_ref());
                raw.extend(r.as_ref());
            }
        }
        bits.clone_from_slice(&raw);
        Ok((HashValue::digest_of(&bits), bits))
    }

    pub fn decode(raw: &[u8]) -> Result<Self> {
        ensure!(raw.len() == 65, "not an encoded node");
        let tag = raw[0];
        let mut left = [0; 32];
        let mut right = [0; 32];
        left.copy_from_slice(&raw[1..33]);
        right.copy_from_slice(&raw[33..]);
        let contents = (HashValue::new(left), HashValue::new(right));
        match tag {
            LEAF_TAG => Ok(Self::Leaf(contents)),
            INTERNAL_TAG => Ok(Self::Internal(contents)),
            _ => Err(anyhow!("Unrecognized node tag")),
        }
    }

    pub fn new_leaf(key: HashValue, value_hash: HashValue) -> Self {
        Node::Leaf((key, value_hash))
    }

    pub fn new_internal(left: HashValue, right: HashValue) -> Self {
        Node::Internal((left, right))
    }

    pub fn is_leaf(&self) -> bool {
        match self {
            Node::Leaf(_) => true,
            _ => false,
        }
    }
}
