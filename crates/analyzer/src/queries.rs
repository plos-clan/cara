use std::sync::{Arc, LazyLock};

use ast::{
    ConstExp, ConstInitialValue, Exp, FileTable, Span,
    visitor::{BlockVisitor, ExpVisitor},
};
use lint::LintDumper;
use query::{DefId, Provider, QueryContext};

use crate::{AnalyzerContext, Error, Symbol, Type, Value, Warning};

pub static CHECK_CONST_DEF: LazyLock<Provider<DefId, AnalyzeResult>> =
    LazyLock::new(|| Provider::new(check_const_def));

#[must_use]
#[derive(Default)]
pub struct AnalyzeResult {
    pub(crate) value: Value,
    pub(crate) errors: Vec<(Error, Span)>,
    pub(crate) warnings: Vec<(Warning, Span)>,
    pub(crate) required: Vec<DefId>,
}

impl AnalyzeResult {
    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warning(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn dump(&mut self, ctx: Arc<QueryContext>, file_table: &FileTable) {
        for required in &self.required {
            let AnalyzeResult {
                errors, warnings, ..
            } = ctx.query(&CHECK_CONST_DEF, *required).unwrap();
            self.errors.extend(errors);
            self.warnings.extend(warnings);
        }

        let mut dumper = LintDumper::new(file_table);

        dumper.lints(self.errors.iter());
        dumper.lints(self.warnings.iter());

        dumper.dump();
    }
}

fn check_const_def(ctx: Arc<QueryContext>, def_id: DefId) -> AnalyzeResult {
    let Some(const_def) = ctx.get_def(def_id) else {
        return AnalyzeResult::default();
    };
    let mut analyzer_ctx = AnalyzerContext::new(ctx.clone(), None);

    let result = match &const_def.initial_value {
        ConstInitialValue::Exp(ConstExp { exp }) => {
            let ast_ctx = ctx.ast_ctx();
            let exp_body = ast_ctx.exp(*exp);
            let ty = match exp_body {
                Exp::ProtoDef(proto) => {
                    let ret_ty = proto
                        .return_type
                        .as_ref()
                        .map(|t| analyzer_ctx.visit_right_value(*t).into_type())
                        .unwrap_or(Type::Unit);
                    let param_types = proto
                        .params
                        .iter()
                        .map(|p| analyzer_ctx.visit_right_value(p.param_type).into_type())
                        .collect::<Vec<_>>();
                    Type::Function(Box::new(ret_ty), param_types)
                }
                Exp::Function(func) => {
                    let ret_ty = func
                        .return_type
                        .as_ref()
                        .map(|t| analyzer_ctx.visit_right_value(*t).into_type())
                        .unwrap_or(Type::Unit);
                    analyzer_ctx.ret_ty = Some(ret_ty.clone());
                    let mut param_types = Vec::new();
                    for param in func.params.iter() {
                        let ty = analyzer_ctx.visit_right_value(param.param_type).into_type();
                        param_types.push(ty.clone());
                        analyzer_ctx.symbols.pre_push(Symbol::Var(
                            param.name.clone(),
                            false,
                            Value::new(ty),
                        ));
                    }
                    if let Some(got_ret_ty) = <AnalyzerContext as BlockVisitor<_>>::visit_block(
                        &mut analyzer_ctx,
                        &func.block,
                    ) {
                        let got_ret_ty = got_ret_ty.into_type();
                        if ret_ty != got_ret_ty {
                            analyzer_ctx.error_at(
                                Error::TypeMismatch(ret_ty.clone(), got_ret_ty),
                                func.block.span,
                            );
                        }
                    }
                    Type::Function(Box::new(ret_ty), param_types)
                }
                _ => analyzer_ctx.visit_right_value(*exp).into_type(),
            };
            Value::new(ty)
        }
    };

    let AnalyzerContext {
        errors,
        warnings,
        required,
        ..
    } = analyzer_ctx;

    AnalyzeResult {
        value: result,
        errors,
        warnings,
        required,
    }
}
