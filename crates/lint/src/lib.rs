use std::fmt::Display;

use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet, renderer::DecorStyle};
use ast::{FileTable, Span};

pub struct LintDumper<'d> {
    file_table: &'d FileTable,
    report: Vec<Group<'d>>,
}

impl<'d> LintDumper<'d> {
    pub fn new(file_table: &'d FileTable) -> Self {
        Self {
            file_table,
            report: Vec::new(),
        }
    }
}

impl LintDumper<'_> {
    pub fn lints<'a, I, T: Display + 'a>(&mut self, lints: I) -> &mut Self
    where
        I: Iterator<Item = &'a (T, Span)>,
    {
        for (error, span) in lints {
            let file = span.file();
            let path = self.file_table.get_path(file).unwrap();
            let source_code = (*self.file_table.read_source(file).unwrap()).clone();

            let error = format!("{}", error);

            self.report.push(
                Level::ERROR.primary_title(error.clone()).element(
                    Snippet::source(source_code).path(path).annotation(
                        AnnotationKind::Primary
                            .span(span.start()..span.end())
                            .label(error),
                    ),
                ),
            );
        }
        self
    }

    pub fn dump(&self) {
        if self.report.is_empty() {
            return;
        }
        let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
        anstream::println!("{}", renderer.render(&self.report));
    }
}
