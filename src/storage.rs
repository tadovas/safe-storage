use crate::merkle;
use crate::sha3::hash_content;

pub struct Content {
    name: String,
    content: Vec<u8>,
}

#[derive(Default)]
pub struct Storage {
    tree: merkle::Sha3Tree,
    files: Vec<Content>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            files: Default::default(),
            tree: merkle::Sha3Tree::new(),
        }
    }

    pub fn add_new_file(&mut self, name: String, content: Vec<u8>) -> usize {
        self.tree.append(hash_content(&content));
        self.files.push(Content { name, content });
        self.files.len() - 1
    }

    pub fn list_all_files(&self) -> Vec<(usize, String, Vec<u8>)> {
        self.files
            .iter()
            .enumerate()
            .map(|(i, v)| (i, v.name.clone(), v.content.clone()))
            .collect()
    }

    pub fn get_file_by_id(&self, id: usize) -> Option<(String, Vec<u8>, merkle::Sha3Proof)> {
        self.files.get(id).map(|c| {
            (
                c.name.clone(),
                c.content.clone(),
                self.tree
                    .proof_for(id)
                    .expect("should be present since we found file with same id"),
            )
        })
    }

    pub fn root_hash(&self) -> Option<merkle::Sha3Hash> {
        self.tree.root()
    }
}
