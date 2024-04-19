#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_braces)]

use crate::ast::{ASTChildren, ASTFlatten, ASTInstanceOf, ASTType, Block, HybridArgumentIdf, AST};
use dyn_clone::DynClone;
// T = TypeVar("T")
// std::marker::Sync +
pub trait TransformerVisitorEx: DynClone + AstTransformerVisitor {
    fn visitBlock(
        &self,
        _ast: Option<ASTFlatten>,
        _guard_cond: Option<HybridArgumentIdf>,
        _guard_val: Option<bool>,
    ) -> Option<ASTFlatten> {
        None
    }
}
dyn_clone::clone_trait_object!(TransformerVisitorEx);
#[derive(Clone)]
pub struct AstTransformerVisitorBase {
    log: bool,
}
impl AstTransformerVisitorBase {
    pub fn new(log: bool) -> Self {
        Self { log }
    }
}
pub trait AstTransformerVisitorBaseRef {
    fn ast_transformer_visitor_base_ref(&self) -> &AstTransformerVisitorBase;
}
pub trait AstTransformerVisitorBaseProperty {
    fn log(&self) -> bool;
}
impl<T: AstTransformerVisitorBaseRef> AstTransformerVisitorBaseProperty for T {
    fn log(&self) -> bool {
        self.ast_transformer_visitor_base_ref().log
    }
}

pub trait AstTransformerVisitor: AstTransformerVisitorBaseProperty {
    fn default() -> Self
    where
        Self: Sized;

    fn visit(&self, ast: &ASTFlatten) -> Option<ASTFlatten> {
        self._visit_internal(ast)
    }
    fn has_attr(&self, name: &ASTType) -> bool;
    fn get_attr(&self, name: &ASTType, ast: &ASTFlatten) -> Option<ASTFlatten>;
    fn visit_list(&self, ast_list: &Vec<ASTFlatten>) -> Vec<ASTFlatten> {
        ast_list.iter().filter_map(|a| self.visit(a)).collect()
    }
    fn visit_children(&self, ast: &ASTFlatten) {
        for c in ast.children() {
            self.visit(&c);
        }
    }
    fn _visit_internal(&self, ast: &ASTFlatten) -> Option<ASTFlatten> {
        if self.log() {
            // std::any::type_name::<Option<String>>(),
            print!("Visiting {:?}", ast);
        }

        self.get_visit_function(ast.get_ast_type(), &ast)
    }

    fn get_visit_function(&self, c: ASTType, ast: &ASTFlatten) -> Option<ASTFlatten> {
        if self.has_attr(&c) {
            self.get_attr(&c, ast)
        } else if let Some(c) = AST::bases(c) {
            self.get_visit_function(c, ast)
        } else {
            None
        }
    }

    fn visitAST(&self, ast: &ASTFlatten) {
        self.visit_children(ast)
    }
}
// class AstTransformerVisitor
// """
// Visitor which replaces visited AST elements by the corresponding visit functions return value

// The default action when no matching visit function is defined, is to replace the node with itself and to visit
// the children. If a matching visit function is defined, children are not automatically visited.
// (Corresponds to node-or-children traversal order from AstVisitor)
// """
