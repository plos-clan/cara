use colored::*;

mod defs;
mod program;
mod types;
mod expr;

pub use defs::*;
pub use program::*;
pub use types::*;
pub use expr::*;

#[derive(Debug, Clone)]
pub struct Span {
    start: (usize,usize),
    end: (usize,usize),
    string: String,
    file: String,
}

impl Span {
    pub fn new(start: (usize,usize), end: (usize,usize), code: String, file: String) -> Self {
        Self {
            start,
            end,
            string: code,
            file,
        }
    }
}

impl Span {
    pub fn show(&self, f: &mut std::fmt::Formatter<'_>, error_string: String) -> std::fmt::Result {
        let num_len = format!("{}", self.start.0).len();
        
        writeln!(f,"{}{} {}","error".red().bold(),":".bold(),error_string.bold())?;
        
        for _ in 0..num_len {
            write!(f, " ")?;
        }
        writeln!(f,"{} {}","-->".blue().bold(),format!("{}:{}:{}",self.file, self.start.0,self.start.1))?;
        
        for _ in 0..=num_len {
            write!(f, " ")?;
        }
        writeln!(f,"{}","|".blue().bold())?;

        write!(f, "{} {} {}",self.start.0,"|".blue().bold() ,self.string)?;

        for _ in 0..=num_len {
            write!(f, " ")?;
        }
        write!(f,"{} ","|".bold().blue())?;

        for _ in 0..self.start.1-1 {
            write!(f, " ")?;
        }

        for _ in self.start.1..self.end.1 {
            write!(f, "{}","^".red().bold())?;
        }
        Ok(())
    }
    
}
