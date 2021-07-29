use anyhow::{anyhow, bail, Result};

use crate::store::MemoryStore;
use crate::types::{HashValue, Node, DEFAULT_VALUE};

pub struct SparseMerkleTree {
    root: HashValue,
    store: MemoryStore,
}

impl SparseMerkleTree {
    pub fn new(root: Option<HashValue>) -> Self {
        Self {
            root: root.unwrap_or(HashValue::placeholder()),
            store: MemoryStore::new(),
        }
    }

    pub fn set_root(&mut self, root: HashValue) {
        self.root = root;
    }

    pub fn get_root(&self) -> HashValue {
        self.root
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        if self.root.is_placeholder() {
            return None;
        }
        self.store.get_value(HashValue::digest_of(key)).ok()
    }

    pub fn update(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let new_root = self.update_for_root(key, value, self.root)?;
        self.set_root(new_root);
        Ok(())
    }

    pub fn update_for_root(
        &mut self,
        key: &[u8],
        value: &[u8],
        root: HashValue,
    ) -> Result<HashValue> {
        let path = HashValue::digest_of(key);
        let (sidenodes, pathnodes, old_leaf_node, _) = self.get_sidenodes(path, root, false)?;

        if value == DEFAULT_VALUE {
            match self.delete_for_sidenode(path, sidenodes, pathnodes, old_leaf_node) {
                Ok(r) => {
                    self.store.delete_value(&path);
                    Ok(r)
                }
                Err(_) => Ok(root),
            }
        } else {
            self.update_with_sidenodes(path, value, sidenodes, pathnodes, old_leaf_node)
        }
    }

    fn update_with_sidenodes(
        &mut self,
        path: HashValue,
        value: &[u8],
        sidenodes: Vec<HashValue>,
        pathnodes: Vec<HashValue>,
        old_leaf_node: Option<Node>,
    ) -> Result<HashValue> {
        let value_hash = HashValue::digest_of(value);
        let node = Node::new_leaf(path, value_hash);

        let current_hash = node.encode().and_then(|(h, d)| self.store.set_node(h, d))?;
        let mut next_hash = current_hash;

        let path_node_root = pathnodes.get(0).ok_or(anyhow!("pathnodes is empty"))?;

        let mut old_value_hash = None;
        let mut common_prefix_count = HashValue::DEPTH;

        if !path_node_root.is_placeholder() {
            let n = old_leaf_node.ok_or(anyhow!("old_leaf_data is None"))?;
            let (actual_path, actual_value_hash) = match n {
                Node::Leaf(content) => content,
                _ => bail!("expected leaf"),
            };
            old_value_hash = Some(actual_value_hash);
            common_prefix_count = path.common_prefix_bits_len(actual_path);
        }

        if common_prefix_count != HashValue::DEPTH {
            let node = match path.has_bit_set(common_prefix_count) {
                // right
                true => Node::new_internal(*path_node_root, next_hash),
                _ => Node::new_internal(next_hash, *path_node_root),
            };

            let current_hash = node.encode().and_then(|(h, d)| self.store.set_node(h, d))?;
            next_hash = current_hash;
        } else if old_value_hash.is_some() {
            let uovh = old_value_hash.unwrap();
            if uovh == value_hash {
                return Ok(self.root);
            }
            self.store.delete_node(&path_node_root);
            self.store.delete_value(&path);
        }

        for index in 1..pathnodes.len() {
            self.store.delete_node(pathnodes.get(index).unwrap());
        }

        for i in 0..HashValue::DEPTH {
            let temp = &HashValue::placeholder();
            let sn = sidenodes.get(i).or_else(|| {
                if common_prefix_count != HashValue::DEPTH
                    && common_prefix_count > HashValue::DEPTH - 1 - i
                {
                    Some(temp)
                } else {
                    None
                }
            });
            if sn.is_none() {
                continue;
            }

            let sidenode = sn.unwrap();
            let node = match path.has_bit_set(common_prefix_count) {
                // go right
                true => Node::new_internal(*sidenode, next_hash),
                _ => Node::new_internal(next_hash, *sidenode),
            };

            let current_hash = node.encode().and_then(|(h, d)| self.store.set_node(h, d))?;
            next_hash = current_hash;
        }

        self.store.set_value(path, value)?;
        Ok(current_hash)
    }

    fn delete_for_sidenode(
        &mut self,
        path: HashValue,
        sidenodes: Vec<HashValue>,
        pathnodes: Vec<HashValue>,
        old_leaf_node: Option<Node>,
    ) -> Result<HashValue> {
        if pathnodes
            .get(0)
            .expect("pathnode should have root")
            .is_placeholder()
        {
            bail!("Key is already empty")
        }

        let n = old_leaf_node.ok_or(anyhow!("old_leaf_data is None"))?;
        let (actual_path, _) = match n {
            Node::Leaf(content) => content,
            _ => bail!("expected leaf"),
        };
        if actual_path != path {
            bail!("Key is already empty");
        }

        for key in &pathnodes {
            self.store.delete_node(key);
        }

        /*
        // TODO: finish
        let mut non_placeholder_reached = false;
        let mut current_hash: &[u8] = b"";
        let mut current_data: &[u8] = b"";
        for (index, snk) in sidenodes.iter().enumerate() {
            if current_data.len() == 0 {
                let sidenode = self
                    .store
                    .get_node(*snk)
                    .and_then(|raw| Node::decode(raw))?;
                if sidenode.is_leaf() {
                    current_hash = snk.as_ref();
                    current_data = snk.as_ref();
                    continue;
                } else {
                    current_data = HashValue::placeholder().as_ref();
                    non_placeholder_reached = true;
                }
            }

            if !non_placeholder_reached && sidenode.is_placeholder() {}
        } */

        Ok(HashValue::placeholder())
    }

    fn get_sidenodes(
        &self,
        path: HashValue,
        root: HashValue,
        siblingdata: bool,
    ) -> Result<(
        Vec<HashValue>,
        Vec<HashValue>,
        Option<Node>,
        Option<Vec<u8>>,
    )> {
        let snodes: Vec<HashValue> = Vec::new();
        let pnodes: Vec<HashValue> = vec![root];

        if root.is_placeholder() {
            return Ok((snodes, pnodes, None, None));
        }

        let node = self
            .store
            .get_node(root)
            .and_then(|raw| Node::decode(raw))?;
        if node.is_leaf() {
            return Ok((snodes, pnodes, Some(node), None));
        }

        let (sidenodes, pathnodes, cd, sibdata) =
            self.walk_for_subnodes(path, snodes, pnodes, node, siblingdata)?;

        Ok((sidenodes, pathnodes, cd, sibdata))
    }

    fn walk_for_subnodes(
        &self,
        path: HashValue,
        mut sidenodes: Vec<HashValue>,
        mut pathnodes: Vec<HashValue>,
        current_node: Node,
        with_sibdata: bool,
    ) -> Result<(
        Vec<HashValue>,
        Vec<HashValue>,
        Option<Node>,
        Option<Vec<u8>>,
    )> {
        let mut node = current_node;

        for i in 0..HashValue::DEPTH {
            let (sidenode, nodehash) = match node {
                Node::Internal((left, right)) => match path.has_bit_set(i) {
                    // go right
                    true => (left, right),
                    _ => (right, left),
                },
                _ => bail!("expected internal node"),
            };

            sidenodes.push(sidenode);
            pathnodes.push(nodehash);

            if nodehash.is_placeholder() {
                sidenodes.reverse();
                pathnodes.reverse();
                return Ok((sidenodes, pathnodes, None, None));
            }

            node = self
                .store
                .get_node(nodehash)
                .and_then(|raw| Node::decode(raw))?;
            if node.is_leaf() {
                sidenodes.reverse();
                pathnodes.reverse();
                return Ok((sidenodes, pathnodes, Some(node), None));
            }
        }

        /*
        if with_sibdata {
            let sibdata = self.store.get_node(sidenode)?;
            sidenodes.reverse();
            pathnodes.reverse();
            return Ok((
                sidenodes,
                pathnodes,
                Some(current_data.clone()),
                Some(sibdata.clone()),
            ));
        }*/

        sidenodes.reverse();
        pathnodes.reverse();
        Ok((sidenodes, pathnodes, Some(node), None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use rand::{Rng, RngCore};

    fn generate_seed() -> Vec<u8> {
        let mut rng = OsRng;
        let k: [u8; 32] = rng.gen();
        k.to_vec()
    }

    fn random_key(alphabet: &[u8], min_count: usize, diff_count: usize) -> Vec<u8> {
        let seed = generate_seed();
        assert!(min_count + diff_count <= 32);
        let r = min_count + (seed[31] as usize % (diff_count + 1));
        let mut ret: Vec<u8> = Vec::with_capacity(r);
        for i in 0..r {
            ret.push(alphabet[seed[i] as usize % alphabet.len()]);
        }
        ret
    }

    fn random_value() -> Vec<u8> {
        let mut v = vec![1; rand::thread_rng().gen_range(10..200)];
        rand::thread_rng().fill_bytes(&mut v);
        v
    }

    #[test]
    fn test_tree() {
        let mut tree = SparseMerkleTree::new(None);

        assert!(tree.get(b"a").is_none());
        assert!(tree.get_root().is_placeholder());
        assert!(tree.update(b"a", b"a1").is_ok());

        assert_eq!(tree.get(b"a").unwrap(), b"a1");
        assert!(!tree.get_root().is_placeholder());

        assert!(tree.update(b"b", b"b1").is_ok());
        assert!(tree.update(b"c", b"c1").is_ok());
        assert!(tree.update(b"d", b"d1").is_ok());
        assert!(tree.update(b"e", b"e1").is_ok());
        assert!(tree.update(b"f", b"f1").is_ok());
        assert!(tree.update(b"g", b"g1").is_ok());

        assert_eq!(tree.get(b"a").unwrap(), b"a1");
        assert_eq!(tree.get(b"b").unwrap(), b"b1");
        assert_eq!(tree.get(b"c").unwrap(), b"c1");
        assert_eq!(tree.get(b"d").unwrap(), b"d1");
        assert_eq!(tree.get(b"e").unwrap(), b"e1");
        assert_eq!(tree.get(b"f").unwrap(), b"f1");

        println!("ROOT: {:x}", tree.get_root());
    }

    #[test]
    fn batch() {
        let alphabet = b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]_";
        let mut d: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

        for _ in 0..100 {
            d.push((random_key(alphabet, 10, 20), random_value()))
        }

        let mut tree = SparseMerkleTree::new(None);
        for (k, v) in &d {
            assert!(tree.update(k, v).is_ok());
        }

        assert!(!tree.get_root().is_placeholder());

        for (k, v) in &d {
            assert_eq!(tree.get(k).unwrap(), v);
        }
    }
}
