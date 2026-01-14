use std::{collections::HashMap, fs::File, io::Read, sync::Arc};

pub use defs::*;
pub use expr::*;
pub use program::*;
pub use types::*;

mod defs;
mod expr;
mod program;
mod types;
pub mod visitor;

pub trait Parser {
    type Error: From<std::io::Error>;

    fn file_table(&self) -> &FileTable;
    fn set_current_file(&mut self, file: usize);
    fn current_file(&self) -> usize;

    fn parse_content(&self, content: &str) -> Result<Type, Self::Error>;

    fn parse(&mut self, file: usize) -> Result<Type, Self::Error> {
        self.set_current_file(file);

        let content = self.file_table().read_source(file).unwrap();
        self.parse_content(content.as_ref())
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span(start, end, self.current_file())
    }
}

pub struct FileTable {
    file_ids: HashMap<String, usize>,
    files: HashMap<usize, (String, Arc<String>)>,
}

impl Default for FileTable {
    fn default() -> Self {
        Self::new()
    }
}

impl FileTable {
    pub fn new() -> Self {
        Self {
            file_ids: HashMap::new(),
            files: HashMap::new(),
        }
    }

    pub fn register_file(&mut self, path: String) -> std::io::Result<usize> {
        if let Some(id) = self.file_ids.get(&path) {
            return Ok(*id);
        }

        let mut content = String::new();
        File::open(path.clone())?.read_to_string(&mut content)?;
        let content = Arc::new(content);

        let id = self.file_ids.len();
        self.file_ids.insert(path.clone(), id);
        self.files.insert(id, (path, content));

        Ok(id)
    }

    pub fn read_source(&self, file: usize) -> Option<Arc<String>> {
        self.files.get(&file).map(|(_, source)| source.clone())
    }

    pub fn get_path(&self, file: usize) -> Option<String> {
        self.files.get(&file).map(|(path, _)| path.clone())
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Span(usize, usize, usize);

impl Span {
    pub fn file(&self) -> usize {
        self.2
    }

    pub fn len(&self) -> usize {
        self.1 - self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == self.1
    }

    pub fn start(&self) -> usize {
        self.0
    }

    pub fn end(&self) -> usize {
        self.1
    }
}
