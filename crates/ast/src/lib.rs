use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    hash::{Hash, Hasher},
    io::Read,
    sync::Arc,
};

pub use defs::*;
pub use expr::*;
pub use program::*;
pub use types::*;

mod defs;
mod expr;
mod program;
mod types;
pub mod visitor;

#[derive(Debug, Clone, Copy)]
pub struct ExpId(u64, Span);

impl ExpId {
    pub fn new(id: u64, span: Span) -> Self {
        ExpId(id, span)
    }

    pub fn span(&self) -> Span {
        self.1
    }
}

impl PartialEq for ExpId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ExpId {}

impl Hash for ExpId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub struct ParseContext<'ctx> {
    file_table: &'ctx FileTable,
    exp_map: RefCell<HashMap<ExpId, Exp>>,
    current_file: usize,
}

impl ParseContext<'_> {
    pub fn insert_exp(&self, exp: Exp) -> ExpId {
        let id = ExpId(self.exp_map.borrow().len() as u64, self.span(0, 0));
        self.exp_map.borrow_mut().insert(id, exp);
        id
    }

    pub fn span(&self, start: usize, end: usize) -> Span {
        Span(start, end, self.current_file)
    }
}

impl<'ctx> ParseContext<'ctx> {
    pub fn new(file_table: &'ctx FileTable, current_file: usize) -> Self {
        ParseContext {
            file_table,
            exp_map: RefCell::new(HashMap::new()),
            current_file,
        }
    }

    pub fn parse<T: Parser>(mut self, parser: T) -> Result<AstContext, T::Error> {
        let content = self.file_table.read_source(self.current_file).unwrap();

        let root = parser.parse_content(&mut self, content)?;
        Ok(AstContext {
            exp_map: self.exp_map.take(),
            root,
        })
    }
}

pub struct AstContext {
    exp_map: HashMap<ExpId, Exp>,
    pub root: StructType,
}

impl AstContext {
    pub fn new(exp_map: HashMap<ExpId, Exp>, root: StructType) -> Self {
        AstContext { exp_map, root }
    }
}

impl AstContext {
    pub fn into_tuple(self) -> (HashMap<ExpId, Exp>, StructType) {
        (self.exp_map, self.root)
    }

    pub fn exp(&self, id: ExpId) -> &Exp {
        self
            .exp_map
            .get(&id)
            .unwrap_or_else(|| panic!("ExpId {:?} not found", id))
    }

    pub fn exp_mut(&mut self, id: ExpId) -> &mut Exp {
        self.exp_map.get_mut(&id).unwrap()
    }
}

pub trait Parser {
    type Error: From<std::io::Error>;

    fn parse_content(
        &self,
        ctx: &mut ParseContext<'_>,
        content: Arc<String>,
    ) -> Result<StructType, Self::Error>;
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
