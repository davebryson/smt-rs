use anyhow::{ensure, Result};
use blake2::{Blake2s, Digest};

pub type HashKey = [u8; 32];

pub const LEAF: u8 = 0;
pub const NODE: u8 = 1;
pub const RIGHT: u8 = 1;
pub const KEY_SIZE: usize = 32;
pub const DEPTH: usize = 256;
pub const DEFAULTVALUE: &[u8; 0] = b"";
pub const PLACEHOLDER: [u8; KEY_SIZE] = [0; KEY_SIZE];

pub fn digest(data: &[u8]) -> [u8; 32] {
    let mut a = [0; KEY_SIZE];
    let mut hasher = Blake2s::new();
    hasher.update(data);
    a.copy_from_slice(hasher.finalize().as_ref());
    a
}

// Leaf is the hashed key and hashed value
pub fn create_leaf(key: HashKey, value: HashKey) -> (HashKey, Vec<u8>) {
    // 65 bytes
    let raw = [&[LEAF], key.as_ref(), value.as_ref()].concat();
    let h = digest(&raw);
    (h, raw)
}

pub fn parse_leaf(data: &[u8]) -> Result<(HashKey, HashKey)> {
    ensure!(is_leaf(data), "not a leaf");
    let mut key: HashKey = [0; 32];
    let mut value: HashKey = [0; 32];
    key.copy_from_slice(&data[1..33]);
    value.copy_from_slice(&data[33..]);
    Ok((key, value))
}

pub fn is_leaf(data: &[u8]) -> bool {
    data[0] == LEAF
}

pub fn create_node(left: &[u8], right: &[u8]) -> (HashKey, Vec<u8>) {
    let raw = [&[NODE], left.as_ref(), right.as_ref()].concat();
    let h = digest(&raw);
    (h, raw)
}

// Left and right nodes are always 32
pub fn parse_node(data: &[u8]) -> Result<(HashKey, HashKey)> {
    ensure!(data[0] == NODE, "not a node");
    let mut left: HashKey = [0; 32];
    let mut right: HashKey = [0; 32];
    left.copy_from_slice(&data[1..33]);
    right.copy_from_slice(&data[33..]);
    Ok((left, right))
}

pub fn get_bit(index: usize, data: &[u8]) -> u8 {
    if data[index >> 3] & 1 << (7 - index % 8) > 0 {
        return 1;
    }
    0
}

// NOT USED?
pub fn count_set_bits(data: &[u8]) -> u16 {
    let mut count: u16 = 0;
    for i in 0..data.len() * 8 {
        if get_bit(i, data) == 1 {
            count += 1;
        }
    }
    count
}

pub fn count_common_prefix(a: &[u8], b: &[u8]) -> usize {
    // Should ensure they are both the same size
    let mut count = 0;
    for i in 0..a.len() * 8 {
        if get_bit(i, a) == get_bit(i, b) {
            count += 1;
        } else {
            return count;
        }
    }
    count
}

#[cfg(test)]
mod tests {

    use super::get_bit;

    pub fn bit(index: usize, data: &[u8]) -> bool {
        let pos = index / 8;
        let bit = 7 - index % 8;
        (data[pos] >> bit) & 1 != 0
    }

    pub fn print_bits(data: &[u8]) {
        for v in data {
            print!("{:#010b}", v)
        }
        println!("")
    }

    #[test]
    fn test_bits() {
        // 0b01100001
        let ex = b"a";

        //print_bits(b"a");
        //print_bits(b"b");
        //print_bits(b"c");
        //print_bits(b"d");

        /*
        let a = HashValue::digest_of(b"a");
        let b = HashValue::digest_of(b"b");
        let c = HashValue::digest_of(b"c");
        let d = HashValue::digest_of(b"d");

        println!("{:#b}", a);
        println!("");
        println!("{:#b}", b);
        println!("");
        println!("{:#b}", c);
        println!("");
        println!("{:#b}", d);
        println!("");

        assert_eq!(1, c.common_prefix_bits_len(d.clone()));
        assert_eq!(1, a.common_prefix_bits_len(b));
        assert_eq!(0, a.common_prefix_bits_len(d));
        */

        assert_eq!(0, get_bit(0, ex));
        assert_eq!(1, get_bit(1, ex));
        assert_eq!(1, get_bit(2, ex));
        assert_eq!(0, get_bit(3, ex));
        assert_eq!(0, get_bit(4, ex));
        assert_eq!(0, get_bit(5, ex));
        assert_eq!(0, get_bit(6, ex));
        assert_eq!(1, get_bit(7, ex));

        assert_eq!(false, bit(0, ex));
        assert_eq!(true, bit(1, ex));
        assert_eq!(true, bit(2, ex));
        assert_eq!(false, bit(3, ex));
        assert_eq!(false, bit(4, ex));
        assert_eq!(false, bit(5, ex));
        assert_eq!(false, bit(6, ex));
        assert_eq!(true, bit(7, ex));
    }
}
