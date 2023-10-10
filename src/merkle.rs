use crate::sha3;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

type HashList<T> = Vec<T>;

pub trait Hash<T> {
    fn hash_of(left: &T, right: &T) -> T;
}

#[derive(Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum NodeState<T> {
    PartialLeft(T),
    PartialRight(T),
    Full,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LightNode<T> {
    hash: T,
    state: NodeState<T>,
}

impl<T> LightNode<T>
where
    T: Clone + Hash<T>,
{
    fn new_node(
        &self,
        hash: T,
        mut new_element_stored: bool,
        full_previous_node: bool,
    ) -> (LightNode<T>, bool) {
        let (state, hash) = match self.state {
            NodeState::PartialLeft(ref left_hash) => {
                assert!(new_element_stored);
                (
                    if full_previous_node {
                        NodeState::PartialRight(hash.clone())
                    } else {
                        NodeState::PartialLeft(left_hash.clone())
                    },
                    T::hash_of(&hash, &hash),
                )
            }
            NodeState::PartialRight(ref left_hash) if !new_element_stored => {
                new_element_stored = true;
                (NodeState::Full, T::hash_of(&left_hash, &hash))
            }

            NodeState::PartialRight(ref left_hash) => (
                if full_previous_node {
                    NodeState::Full
                } else {
                    NodeState::PartialRight(left_hash.clone())
                },
                T::hash_of(&left_hash, &hash),
            ),

            NodeState::Full if !new_element_stored => {
                new_element_stored = true;
                (
                    NodeState::PartialRight(hash.clone()),
                    T::hash_of(&hash, &hash),
                )
            }

            NodeState::Full => (
                NodeState::PartialLeft(hash.clone()),
                T::hash_of(&hash, &hash),
            ),
        };

        (LightNode { state, hash }, new_element_stored)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LightTree<T>
where
    T: Debug + PartialEq,
{
    nodes: Vec<LightNode<T>>,
}
impl<T> LightTree<T>
where
    T: Debug + PartialEq,
{
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn append(&mut self, elem: T)
    where
        T: Clone + Hash<T>,
    {
        if self.nodes.is_empty() {
            self.nodes.push(LightNode {
                hash: T::hash_of(&elem, &elem),
                state: NodeState::PartialRight(elem),
            });
            return;
        }

        if let Some(hash) = self
            .nodes
            .last()
            .filter(|node| node.state == NodeState::Full)
            .map(|node| &node.hash)
        {
            // fully filled node - add additional partial on top to save hash of full root node
            self.nodes.push(LightNode {
                hash: hash.clone(),
                state: NodeState::PartialRight(hash.clone()),
            })
        }

        let mut next_hash = elem;
        let mut new_element_stored = false;
        let mut full_previous_node = false;
        for node in self.nodes.iter_mut() {
            let (new_node, stored) =
                node.new_node(next_hash.clone(), new_element_stored, full_previous_node);
            new_element_stored = stored;
            *node = new_node;
            full_previous_node = node.state == NodeState::Full;
            next_hash = node.hash.clone();
        }
    }

    pub fn root(&self) -> Option<T>
    where
        T: Clone + Hash<T>,
    {
        self.nodes.last().map(|node| node.hash.clone())
    }
}

impl<T> Default for LightTree<T>
where
    T: Debug + PartialEq,
{
    fn default() -> Self {
        Self::new()
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

pub type Sha3LightTree = LightTree<Sha3Hash>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::sha3::hash_content;
    use std::u64;

    impl Hash<i32> for i32 {
        fn hash_of(left: &i32, right: &i32) -> i32 {
            *left + *right
        }
    }

    impl Hash<u64> for u64 {
        fn hash_of(left: &u64, right: &u64) -> u64 {
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
        assert!(proof.verify(&root, &50_000))
    }

    #[test]
    pub fn test_lightweight_tree_proof() {
        let mut tree = Tree::new();
        let mut light_tree = LightTree::new();

        // for each added element tree and light_tree root nodes need to be equal
        for i in 0..=16 {
            let value = (i as u64 + 1) * u64::pow(10, i);
            tree.append(value);
            light_tree.append(value);

            println!("After {value} addition");
            println!("{light_tree:#?}");

            assert_eq!(
                tree.root(),
                light_tree.root(),
                "comparing at {i} iteration after value {value} insertion"
            );
        }
    }

    #[test]
    #[ignore = "Super naive m tree vs light tree size comparision"]
    pub fn size_comparision() {
        let mut tree = Sha3Tree::new();
        let mut light_tree = Sha3LightTree::new();

        for i in 0..100000u64 {
            let hash = hash_content(i.to_be_bytes().as_slice());
            tree.append(hash.clone());
            light_tree.append(hash.clone());

            assert_eq!(tree.root(), light_tree.root());
        }

        let tree_bytes = serde_json::to_string(&tree).expect("should serialize");
        let light_tree_bytes = serde_json::to_string(&light_tree).expect("should serialize");

        println!(
            "Size difference is: {}",
            tree_bytes.len() / light_tree_bytes.len()
        )
    }
}
