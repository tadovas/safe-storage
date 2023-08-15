use crate::sha3;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

type HashList<T> = Vec<T>;

pub trait Hash<T> {
    fn hash_of(left: &T, right: &T) -> T;
}

pub struct Tree<T> {
    leaves: HashList<T>,
    nodes: Vec<HashList<T>>,
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self {
            leaves: Default::default(),
            nodes: Default::default(),
        }
    }

    pub fn root(&self) -> Option<T>
    where
        T: Clone,
    {
        self.nodes.last().and_then(|top| top.last().cloned())
    }

    pub fn append(&mut self, hash: T)
    where
        T: Clone,
        T: Hash<T>,
    {
        self.leaves.push(hash.clone());
        let (hashed, right_child_added) = hash_of_siblings(&self.leaves);
        self.update_next_layer(0, hashed, right_child_added);
    }

    fn update_next_layer(&mut self, layer: usize, hash: T, update_last_hash: bool)
    where
        T: Hash<T>,
    {
        let hash_list = self.nodes.get_mut(layer);
        if hash_list.is_none() {
            // special case - if we have a hash and there is no current layer, that means we reached top and
            // hash is new root hash
            let hash_list = vec![hash];
            self.nodes.push(hash_list);
            return;
        }
        // we can safely unwrap here since we checked for none above
        let hash_list: &mut HashList<T> = hash_list.unwrap();
        if update_last_hash {
            let last_idx = hash_list.len() - 1;
            hash_list[last_idx] = hash
        } else {
            hash_list.push(hash);
        }
        if hash_list.len() == 1 {
            return;
        }

        let (hashed, right_child_added) = hash_of_siblings(hash_list);
        self.update_next_layer(layer + 1, hashed, right_child_added || update_last_hash);
    }

    pub fn proof_for(&self, mut index: usize) -> Option<Proof<T>>
    where
        T: Clone + Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let direct_sibling = proof_node_with_sibling(&self.leaves, index);

        let mut proof_nodes = vec![direct_sibling];
        for layer in &self.nodes {
            if layer.len() == 1 {
                break;
            }

            index /= 2;
            proof_nodes.push(proof_node_with_sibling(layer, index));
        }

        Some(Proof { nodes: proof_nodes })
    }
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn hash_of_siblings<T>(hash_list: &HashList<T>) -> (T, bool)
where
    T: Hash<T>,
{
    let right_child_exists = hash_list.len() % 2 == 0;

    let last = hash_list.len() - 1;

    let (left, right) = if right_child_exists {
        let left = &hash_list[last - 1];
        let right = &hash_list[last];
        (left, right)
    } else {
        let last = &hash_list[last];
        (last, last)
    };

    (T::hash_of(left, right), right_child_exists)
}

fn proof_node_with_sibling<T>(hash_list: &HashList<T>, index: usize) -> ProofNode<T>
where
    T: Clone + Debug + PartialEq + Serialize + DeserializeOwned,
{
    let sibling_is_on_the_right = index % 2 == 0;

    if sibling_is_on_the_right {
        hash_list.get(index + 1)
    } else {
        hash_list.get(index - 1)
    }
    .map(|h| match sibling_is_on_the_right {
        true => ProofNode::RightSiblign(h.clone()),
        false => ProofNode::LeftSibling(h.clone()),
    })
    .unwrap_or(ProofNode::None)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ProofNode<T>
where
    T: Debug,
    T: PartialEq,
{
    None,
    RightSiblign(T),
    LeftSibling(T),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Proof<T>
where
    T: Debug + PartialEq,
{
    nodes: Vec<ProofNode<T>>,
}

impl<T> Proof<T>
where
    T: Debug + PartialEq,
{
    pub fn verify(&self, root_hash: &T, hash: &T) -> bool
    where
        T: Hash<T> + Clone,
    {
        let calculated_root = self.nodes.iter().fold(hash.clone(), |h, node| match *node {
            ProofNode::None => T::hash_of(&h, &h),
            ProofNode::RightSiblign(ref right_sibling_hash) => T::hash_of(&h, right_sibling_hash),
            ProofNode::LeftSibling(ref left_sibling_hash) => T::hash_of(left_sibling_hash, &h),
        });
        *root_hash == calculated_root
    }
}

impl Hash<sha3::Hash> for sha3::Hash {
    fn hash_of(left: &sha3::Hash, right: &sha3::Hash) -> sha3::Hash {
        sha3::hash_both(left, right)
    }
}

pub type Sha3Hash = sha3::Hash;
pub type Sha3Tree = Tree<Sha3Hash>;
pub type Sha3Proof = Proof<Sha3Hash>;

#[cfg(test)]
mod test {
    use super::*;

    impl Hash<i32> for i32 {
        fn hash_of(left: &i32, right: &i32) -> i32 {
            *left + *right
        }
    }

    #[test]
    pub fn test_root_of_single_item() {
        let mut tree = Tree::new();
        assert!(tree.root().is_none());

        tree.append(1);
        // if there is only one elmenet - root is hash of element with itself
        assert_eq!(tree.root(), Some(2))
    }

    #[test]
    pub fn test_root_of_two_items() {
        let mut tree = Tree::new();
        assert!(tree.root().is_none());

        tree.append(1);
        tree.append(2);

        assert_eq!(tree.root(), Some(3))
    }

    #[test]
    pub fn test_power_of_two_tree() {
        let mut tree = Tree::new();
        assert!(tree.root().is_none());

        tree.append(1);
        tree.append(20);
        tree.append(300);
        tree.append(4000);
        tree.append(50000);
        tree.append(600000);
        tree.append(7000000);
        tree.append(80000000);

        assert_eq!(tree.root(), Some(87654321))
    }

    #[test]
    pub fn test_root_of_multiple_items() {
        let mut tree = Tree::new();
        assert!(tree.root().is_none());

        tree.append(1);
        tree.append(20);
        tree.append(300);
        tree.append(4000);
        tree.append(50000);

        // layer: 0 (1 + 20 ) (300 + 4000) ( 50 000 + 50 000)
        // layer: 1 (21 + 4300) (100 000 + 100 000)
        // layer: 2 (4321 + 200 000)
        // layer: 3 (204321)

        assert_eq!(tree.root(), Some(204321))
    }

    #[test]
    pub fn test_generated_proof() {
        let mut tree = Tree::new();
        assert!(tree.root().is_none());

        tree.append(10);
        tree.append(200);

        assert_eq!(tree.root(), Some(210));

        let proof = tree.proof_for(1).expect("should be present");
        assert_eq!(vec![ProofNode::LeftSibling(10)], proof.nodes)
    }

    #[test]
    pub fn test_proof_verification() {
        let mut tree = Tree::new();
        assert!(tree.root().is_none());

        tree.append(1);
        tree.append(20);
        tree.append(300);
        tree.append(4_000);
        tree.append(50_000);
        tree.append(600_000);
        tree.append(7_000_000);
        tree.append(80_000_000);

        assert_eq!(tree.root(), Some(87654321));
        let root = tree.root().expect("should exist");

        let proof = tree.proof_for(4).expect("should exist");
        assert!(proof.verify(&root, &50000))
    }
}
