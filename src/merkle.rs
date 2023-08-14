type HashList<T> = Vec<T>;

type ProofList<T> = Vec<Option<T>>;

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

    pub fn proof_for_nth(&self, _index: usize) -> Vec<Option<T>> {
        Default::default()
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
}
