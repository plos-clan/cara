use std::fmt::Display;

use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet, renderer::DecorStyle};
use ast::Span;
use thiserror::Error;

use crate::Type;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid type cast: {0} -> {1}")]
    InvalidTypeCast(Type, Type),
    #[error("Dereferencing value with type {0}.")]
    WrongDeref(Type),
    #[error("Calling value with type {0}.")]
    WrongCall(Type),
    #[error("Type mismatch: Expected {0}, found {1}")]
    TypeMismatch(Type, Type),
    #[error("Unsupported operator {0} for type {1}")]
    UnsupportedOperator(String, Type),
    #[error("{0}")]
    Custom(String),
}

#[derive(Debug, Error)]
pub enum Warning {
    #[error("{0}")]
    Custom(String),
}

pub(crate) struct DiagnosticDumper<'d> {
    source_code: &'d str,
    path: &'d str,
    report: Vec<Group<'d>>,
}

impl<'d> DiagnosticDumper<'d> {
    pub fn new(source_code: &'d str, path: &'d str) -> Self {
        Self {
            source_code,
            path,
            report: Vec::new(),
        }
    }
}

impl DiagnosticDumper<'_> {
    pub fn add_iter<'a, I, T: Display + 'a>(&mut self, iter: I)
    where
        I: Iterator<Item = &'a (T, Span)>,
    {
        for (error, span) in iter {
            let error = format!("{}", error);

            self.report.push(
                Level::ERROR.primary_title(error.clone()).element(
                    Snippet::source(self.source_code)
                        .path(self.path)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.start()..span.end())
                                .label(error),
                        ),
                ),
            );
        }
    }

    pub fn dump(self) {
        let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
        anstream::println!("{}", renderer.render(&self.report));
    }
}
