//!
//! Store stuff
//!

use crate::types::{EncodedNode, HashValue};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct MemoryStore {
    nodes: HashMap<HashValue, Vec<u8>>,
    values: HashMap<HashValue, Vec<u8>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn get_value(&self, key: HashValue) -> Result<&Vec<u8>> {
        self.values.get(&key).ok_or(anyhow!("Invalid Key"))
    }

    pub fn set_value(&mut self, key: HashValue, value: &[u8]) -> anyhow::Result<()> {
        self.values.insert(key, value.to_vec());
        Ok(())
    }

    pub fn delete_value(&mut self, key: &HashValue) -> Option<Vec<u8>> {
        self.values.remove(key)
    }

    // TODO: Actually return the node??
    pub fn get_node(&self, key: HashValue) -> Result<&Vec<u8>> {
        self.nodes.get(&key).ok_or(anyhow!("Invalid Key"))
    }

    // TODO: Take the node as a parameter
    pub fn set_node(&mut self, key: HashValue, value: EncodedNode) -> anyhow::Result<HashValue> {
        self.nodes.insert(key, value.to_vec());
        Ok(key)
    }

    pub fn delete_node(&mut self, key: &HashValue) -> Option<Vec<u8>> {
        self.nodes.remove(key)
    }
}
