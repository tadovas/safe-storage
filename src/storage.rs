use std::sync::Mutex;

pub struct Content {
    name: String,
    content: Vec<u8>,
}

#[derive(Default)]
pub struct Storage {
    files: Mutex<Vec<Content>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            files: Mutex::new(Default::default()),
        }
    }

    pub fn add_new_file(&self, name: String, content: Vec<u8>) -> usize {
        let mut files = self.files.lock().expect("should lock");
        files.push(Content { name, content });
        files.len() - 1
    }

    pub fn list_all_files(&self) -> Vec<(usize, String, Vec<u8>)> {
        let files = self.files.lock().expect("should lock");
        files
            .iter()
            .enumerate()
            .map(|(i, v)| (i, v.name.clone(), v.content.clone()))
            .collect()
    }

    pub fn get_file_by_id(&self, id: usize) -> Option<(String, Vec<u8>)> {
        let files = self.files.lock().expect("should lock");
        files.get(id).map(|c| (c.name.clone(), c.content.clone()))
    }
}
