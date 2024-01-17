use crate::zkay_ast::ast::AST;

// T = TypeVar("T")

pub struct AstTransformerVisitorBase {
    log: bool,
}
pub trait  AstTransformerVisitor{
// type Return ;
// type AST;
 fn default()->Self;
}
// class AstTransformerVisitor
// """
// Visitor which replaces visited AST elements by the corresponding visit functions return value

// The default action when no matching visit function is defined, is to replace the node with itself and to visit
// the children. If a matching visit function is defined, children are not automatically visited.
// (Corresponds to node-or-children traversal order from AstVisitor)
// """

impl AstTransformerVisitor for AstTransformerVisitorBase {
}
impl  AstTransformerVisitorBase {
   
 pub fn new(log: bool) -> Self {
        Self { log }
    }
    pub fn visit(self, ast: AST)->AST {
        self._visit_internal(ast)
    }

    pub fn visit_list(self, ast_list: Vec<AST>)->Vec<AST> {
        ast_list.iter().filter_map(|a| self.visit(a) ).collect()
    }

    pub fn visit_children<T>(self, mut ast: T) -> T {
        ast.process_children(self.visit);
        ast
    }

    pub fn _visit_internal(self, ast: AST)->AST {
        if ast==AST::None {
            return ast
        }

        if self.log {
            println!("Visiting {:?}", ast);
        }
        self.get_visit_function(ast)
    }

    pub fn get_visit_function(self, c: AST)->AST {
        // let visitor_function = "visit" + c.name();
        // if hasattr(self, visitor_function) {
        //     return getattr(self, visitor_function);
        // } else {
        //     for base in c.bases() {
        //         let f = self.get_visit_function(base);
        //         if f.is_some() {
        //             return f;
        //         }
        //     }
        // }
        // assert!(false);
        c
    }

    pub fn visitAST(self, ast: AST)->AST {
        self.visit_children(ast)
    }
}
