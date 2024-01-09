use crate::zkay_ast::ast::{Parameter, SourceUnit};
use crate::zkay_ast::visitor::visitor::AstVisitor;

// class FunctionVisitor(AstVisitor)
pub struct FunctionVisitor;
impl FunctionVisitor {
    // pub fn __init__(self)
    //     super().__init__('node-or-children')

    pub fn visitSourceUnit(&self, ast: SourceUnit) {
        for c in ast.contracts {
            for cd in c.constructor_definitions {
                self.visit(cd);
            }
            for fd in c.function_definitions {
                self.visit(fd);
            }
        }
    }

    pub fn visitParameter(&self, ast: Parameter) {}
}
