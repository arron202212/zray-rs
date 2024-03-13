#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_braces)]

use crate::ast::{
    ASTBaseMutRef, ASTBaseProperty, ASTChildren, ConstructorOrFunctionDefinition, Expression,
    ExpressionBaseMutRef, Identifier, IntoAST, NamespaceDefinition,
    NamespaceDefinitionBaseProperty, SourceUnit, Statement, AST,ASTType
};
use crate::visitor::visitor::AstVisitor;

struct ParentSetterVisitor {
    traversal: String,
}

impl AstVisitor for ParentSetterVisitor {
    type Return = Option<String>;
    fn temper_result(&self) -> Self::Return {
        None
    }
    fn log(&self) -> bool {
        false
    }
    fn traversal(&self) -> &'static str {
        "node-or-children"
    }
    fn has_attr(&self, name: &ASTType) -> bool{
        false
    }
    fn get_attr(&self, name: &ASTType, ast: &AST) -> Option<Self::Return> {
        None
    }
    
}
// class ParentSetterVisitor(AstVisitor)
//     """
//     Links parents
//     """
impl ParentSetterVisitor {
    pub fn new() -> Self {
        Self {
            traversal: String::from("pre"),
        }
    }

    //     pub fn __init__(self)
    //         super().__init__(traversal='pre')

    pub fn visitSourceUnit(&self, ast: &mut SourceUnit) {
        ast.ast_base.namespace = Some(vec![]);
    }

    pub fn visitNamespaceDefinition(&self, mut ast: NamespaceDefinition) {
        ast.ast_base_mut_ref().namespace = Some(if let Some(parent) = ast.parent() {
            parent
                .ast_base_ref()
                .unwrap()
                .namespace
                .as_ref()
                .unwrap()
                .iter()
                .cloned()
                .chain([ast.idf().clone()])
                .collect()
        } else {
            vec![ast.idf().clone()]
        });
    }

    pub fn visitConstructorOrFunctionDefinition(&self, ast: &mut ConstructorOrFunctionDefinition) {
        ast.namespace_definition_base.ast_base.namespace =
            Some(if let Some(parent) = &ast.parent {
                parent
                    .namespace_definition_base
                    .ast_base
                    .namespace
                    .as_ref()
                    .unwrap()
                    .into_iter()
                    .chain([&ast.namespace_definition_base.idf.clone()])
                    .cloned()
                    .collect()
            } else {
                vec![ast.namespace_definition_base.idf.clone()]
            });
    }

    pub fn visitChildren(&self, ast: &mut AST) {
        for c in ast.children().iter_mut() {
            c.ast_base_mut_ref().unwrap().parent = Some(Box::new(ast.clone()));
            c.ast_base_mut_ref().unwrap().namespace = ast.ast_base_ref().unwrap().namespace.clone();
            self.visit(c.clone());
        }
    }
}

struct ExpressionToStatementVisitor;

impl AstVisitor for ExpressionToStatementVisitor {
    type Return = Option<String>;
    fn temper_result(&self) -> Self::Return {
        None
    }
    fn log(&self) -> bool {
        false
    }
    fn traversal(&self) -> &'static str {
        "node-or-children"
    }
    fn has_attr(&self, name: &ASTType) -> bool{
        false
    }
    fn get_attr(&self, name: &ASTType, ast: &AST) -> Option<Self::Return> {
        None
    }
    
}
// class ExpressionToStatementVisitor(AstVisitor)

impl ExpressionToStatementVisitor {
    pub fn visitExpression(&self, ast: &mut Expression) {
        let mut parent = Some(ast.to_ast());
        while let Some(p) = &parent {
            if let AST::Statement(_) = p {
                break;
            }
            parent = p
                .ast_base_ref()
                .unwrap()
                .parent
                .as_ref()
                .map(|p| *p.clone());
        }
        if parent.is_some() {
            ast.expression_base_mut_ref().statement =
                parent.map(|p| Box::new(p.try_as_statement().unwrap()));
        }
    }

    pub fn visitStatement(&self, ast: &mut Statement) {
        let mut parent = Some(ast.to_ast());
        while let Some(p) = &parent {
            if let AST::NamespaceDefinition(NamespaceDefinition::ConstructorOrFunctionDefinition(
                _,
            )) = p
            {
                break;
            }
            parent = p
                .ast_base_ref()
                .unwrap()
                .parent
                .as_ref()
                .map(|p| *p.clone());
        }
        if parent.is_some() {
            ast.statement_base_mut_ref().unwrap().function = parent.map(|p| {
                Box::new(
                    p.try_as_namespace_definition()
                        .unwrap()
                        .try_as_constructor_or_function_definition()
                        .unwrap(),
                )
            });
        }
    }
}

pub fn set_parents(ast: AST) {
    let v = ParentSetterVisitor::new();
    v.visit(ast.clone());
    let v = ExpressionToStatementVisitor;
    v.visit(ast);
}
