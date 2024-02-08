// """
// This module defines zkay->solidity transformers for the smaller contract elements (statements, expressions, state variables).
// """

use circuit_helper::circuit_helper::CircuitHelper;
use regex::Regex;
use regex::RegexSetBuilder;
use solidity::fake_solidity_generator::{ID_PATTERN, WS_PATTERN};
use zkay_ast::analysis::contains_private_checker::contains_private_expr;
use zkay_ast::ast::{
    is_instance, ASTChildren, ASTType, AnnotatedTypeName, AssignmentStatement, BlankLine, Block,
    BooleanLiteralExpr, BooleanLiteralType, BreakStatement, BuiltinFunction, ChildListBuilder,
    Comment, CommentBase, ContinueStatement, DoWhileStatement, ElementaryTypeName,
    EncryptionExpression, EnumDefinition, ExprUnion, Expression, ForStatement, FunctionCallExpr,
    FunctionCallExprBase, HybridArgType, HybridArgumentIdf, Identifier, IdentifierBase,
    IdentifierDeclaration, IdentifierExpr, IdentifierExprUnion, IfStatement, IndexExpr, IntoAST,
    IntoExpression, IntoStatement, LiteralExpr, LocationExpr, Mapping, MeExpr, MemberAccessExpr,
    NamespaceDefinition, NumberLiteralExpr, NumberLiteralType, NumberTypeName, Parameter,
    PrimitiveCastExpr, ReclassifyExpr, ReturnStatement, SimpleStatement, StateVariableDeclaration,
    Statement, StatementList, TupleExpr, TypeName, VariableDeclaration,
    VariableDeclarationStatement, WhileStatement, AST,
};
use zkay_ast::homomorphism::Homomorphism;
use zkay_ast::visitor::deep_copy::replace_expr;
use zkay_ast::visitor::transformer_visitor::{AstTransformerVisitor, TransformerVisitorEx};
use zkay_config::config::CFG;

// class ZkayVarDeclTransformer(AstTransformerVisitor)
// """
// Transformer for types, which was left out in the paper.

// This removes all privacy labels and converts the types of non-public variables (not @all)
// to cipher_type.
// """

// pub fn __init__(self)
//     super().__init__()
//     self.expr_trafo = ZkayExpressionTransformer(None)
#[derive(Clone)]
pub struct ZkayVarDeclTransformer {
    expr_trafo: Option<ZkayExpressionTransformer>,
}
impl AstTransformerVisitor for ZkayVarDeclTransformer {
    fn default() -> Self {
        Self::new()
    }

    fn visit(&self, ast: Option<AST>) -> Option<AST> {
        // self._visit_internal(ast)
        None
    }
    fn visitBlock(
        &self,
        ast: Option<AST>,
        guard_cond: Option<HybridArgumentIdf>,
        guard_val: Option<bool>,
    ) -> Option<AST> {
        // self.visit_children(ast)
        None
    }
}
impl ZkayVarDeclTransformer {
    pub fn new() -> Self {
        Self { expr_trafo: None }
    }

    pub fn visitAnnotatedTypeName(&self, ast: AnnotatedTypeName) -> AnnotatedTypeName {
        let t = if ast.is_private() {
            Some(TypeName::cipher_type(ast.clone(), ast.homomorphism.clone()))
        } else {
            if let Some(AST::TypeName(t)) = self.visit(Some((*ast.type_name).to_ast())) {
                Some(t)
            } else {
                None
            }
        };
        AnnotatedTypeName::new(t.unwrap(), None, String::from("NON_HOMOMORPHISM"))
    }

    pub fn visitVariableDeclaration(&self, ast: &mut VariableDeclaration) -> AST {
        if ast.identifier_declaration_base.annotated_type.is_private() {
            ast.identifier_declaration_base.storage_location = Some(String::from("memory"));
        }
        self.visit_children(Some(ast.to_ast())).unwrap()
    }

    pub fn visitParameter(&self, mut ast: &mut Parameter) -> Option<AST> {
        if let Some(AST::IdentifierDeclaration(IdentifierDeclaration::Parameter(mut ast))) =
            self.visit_children(Some(ast.to_ast()))
        {
            if !ast
                .identifier_declaration_base
                .annotated_type
                .type_name
                .is_primitive_type()
            {
                ast.identifier_declaration_base.storage_location = Some(String::from("memory"));
            }
            Some(ast.to_ast())
        } else {
            None
        }
    }

    pub fn visitStateVariableDeclaration(&self, ast: &mut StateVariableDeclaration) -> AST {
        ast.identifier_declaration_base.keywords = ast
            .identifier_declaration_base
            .keywords
            .iter()
            .filter_map(|k| if k != "public" { Some(k.clone()) } else { None })
            .collect();
        //make sure every state var gets a public getter (required for simulation)
        ast.identifier_declaration_base
            .keywords
            .push(String::from("public"));
        ast.expr = if let Some(AST::Expression(expr)) = self
            .expr_trafo
            .as_ref()
            .unwrap()
            .visit(Some(ast.expr.as_ref().unwrap().to_ast()))
        {
            Some(expr)
        } else {
            None
        };
        self.visit_children(Some(ast.to_ast())).unwrap()
    }

    pub fn visitMapping(&self, ast: &mut Mapping) -> AST {
        if ast.key_label.is_some() {
            ast.key_label = Some(Identifier::Identifier(IdentifierBase::new(
                ast.key_label.as_ref().unwrap().name(),
            )));
        }
        self.visit_children(Some(ast.to_ast())).unwrap()
    }
}
// class ZkayStatementTransformer(AstTransformerVisitor)
// """Corresponds to T from paper, (with additional handling of return statement and loops)."""
#[derive(Clone)]
pub struct ZkayStatementTransformer {
    gen: Option<Box<CircuitHelper>>,
    expr_trafo: ZkayExpressionTransformer,
    var_decl_trafo: ZkayVarDeclTransformer,
}
impl AstTransformerVisitor for ZkayStatementTransformer {
    fn default() -> Self {
        Self::new(None)
    }

    fn visit(&self, ast: Option<AST>) -> Option<AST> {
        // self._visit_internal(ast)
        None
    }
    fn visitBlock(
        &self,
        ast: Option<AST>,
        guard_cond: Option<HybridArgumentIdf>,
        guard_val: Option<bool>,
    ) -> Option<AST> {
        // self.visit_children(ast)
        None
    }
}
impl ZkayStatementTransformer {
    // pub fn __init__(&self, current_gen: CircuitHelper)
    //     super().__init__()
    //     self.gen.unwrap() = current_gen
    //     self.expr_trafo = ZkayExpressionTransformer(self.gen.unwrap())
    //     self.var_decl_trafo = ZkayVarDeclTransformer()
    pub fn new(current_gen: Option<Box<CircuitHelper>>) -> Self {
        Self {
            gen: current_gen.clone(),
            expr_trafo: ZkayExpressionTransformer::new(current_gen),
            var_decl_trafo: ZkayVarDeclTransformer::new(),
        }
    }
    pub fn visitStatementList(&self, ast: &mut StatementList) -> AST
// """
    // Rule (1)

    // All statements are transformed individually.
    // Whenever the transformation of a statement requires the introduction of additional statements
    // (the CircuitHelper indicates this by storing them in the statement"s pre_statements list), they are prepended to the transformed
    // statement in the list.

    // If transformation changes the appearance of a statement (apart from type changes),
    // the statement is wrapped in a comment block which displays the original statement"s code.
    // """
    {
        let mut new_statements = vec![];
        for (idx, stmt) in ast.statements().iter().enumerate() {
            let old_code = stmt.to_ast().code();
            let transformed_stmt = self.visit(Some(stmt.to_ast()));
            if transformed_stmt.is_none() {
                continue;
            }
            let r = Regex::new(r"@{WS_PATTERN}*{ID_PATTERN}")
                .unwrap()
                .replace_all(&old_code, "");
            let old_code_wo_annotations = Regex::new(r"(?=\b)me(?=\b)")
                .unwrap()
                .replace_all(&r, "msg.sender");
            let code = transformed_stmt.as_ref().unwrap().code();
            let new_code_wo_annotation_comments = Regex::new(r"/\*.*?\*/")
                .unwrap()
                .replace_all(code.as_str(), "");
            if old_code_wo_annotations == new_code_wo_annotation_comments {
                new_statements.push(transformed_stmt.unwrap())
            } else {
                new_statements.extend(CommentBase::comment_wrap_block(
                    old_code,
                    transformed_stmt
                        .as_ref()
                        .unwrap()
                        .pre_statements()
                        .iter()
                        .cloned()
                        .chain([transformed_stmt.unwrap()])
                        .collect(),
                ));
            }
        }
        if !new_statements.is_empty()
            && is_instance(new_statements.last().unwrap(), ASTType::BlankLine)
        {
            new_statements.pop();
        }
        ast.set_statements(new_statements);
        ast.to_ast()
    }

    pub fn process_statement_child(&self, child: AST) -> Option<AST>
// """Default statement child handling. Expressions and declarations are visited by the corresponding transformers."""
    {
        if is_instance(&child, ASTType::ExpressionBase) {
            self.expr_trafo.visit(Some(child))
        } else {
            assert!(is_instance(&child, ASTType::VariableDeclaration));
            self.var_decl_trafo.visit(Some(child))
        }
    }

    pub fn visitStatement(&self, ast: &mut Statement) -> AST
// """
    // Rules (3), (4)

    // This is for all the statements where the statements themselves remain untouched and only the children are altered.
    // """
    {
        assert!(
            is_instance(&*ast, ASTType::SimpleStatementBase)
                || is_instance(&*ast, ASTType::VariableDeclarationStatement)
        );
        let mut cb = ChildListBuilder::new();
        ast.process_children(&mut cb);
        cb.children.iter_mut().for_each(|c| {
            self.process_statement_child(c.clone());
        });
        ast.to_ast()
    }

    pub fn visitAssignmentStatement(&mut self, ast: &mut AssignmentStatement) -> AST
// """Rule (2)"""
    {
        let a: AST = self.expr_trafo.visit(ast.lhs().unwrap().into()).unwrap();
        ast.set_lhs(Some(a));
        ast.set_rhs(
            self.expr_trafo
                .visit(Some(ast.rhs().unwrap().to_ast()))
                .unwrap()
                .expr(),
        );
        let mut modvals = ast.modified_values();
        if CFG.lock().unwrap().user_config.opt_cache_circuit_outputs()
            && is_instance(&ast.lhs().unwrap(), ASTType::IdentifierExpr)
            && is_instance(&ast.rhs().unwrap(), ASTType::MemberAccessExpr)
        {
            //Skip invalidation if rhs is circuit output
            if is_instance(
                &ast.rhs().unwrap().member().unwrap(),
                ASTType::HybridArgumentIdf,
            ) && ast.rhs().unwrap().member().unwrap().arg_type() == HybridArgType::PubCircuitArg
            {
                modvals = modvals
                    .iter()
                    .filter_map(|mv| {
                        if mv.target() != ast.lhs().unwrap().location_expr().unwrap().target() {
                            Some(mv.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                let ridf = if is_instance(
                    &ast.rhs()
                        .unwrap()
                        .member()
                        .unwrap()
                        .corresponding_priv_expression()
                        .unwrap(),
                    ASTType::EncryptionExpression,
                ) {
                    ast.rhs()
                        .unwrap()
                        .member()
                        .unwrap()
                        .corresponding_priv_expression()
                        .unwrap()
                        .idf()
                } else {
                    ast.rhs()
                        .unwrap()
                        .member()
                        .unwrap()
                        .corresponding_priv_expression()
                        .unwrap()
                        .idf()
                };
                assert!(is_instance(
                    ridf.as_ref().unwrap(),
                    ASTType::HybridArgumentIdf
                ));
                if let Some(Identifier::HybridArgumentIdf(ridf)) = ridf {
                    self.gen.as_mut().unwrap()._remapper.0.remap(
                        ast.lhs()
                            .unwrap()
                            .location_expr()
                            .unwrap()
                            .target()
                            .unwrap()
                            .idf()
                            .unwrap(),
                        ridf,
                    );
                }
            }
        }

        if self.gen.is_some()
        //Invalidate circuit value for assignment targets
        {
            for val in modvals {
                if val.key().is_none() {
                    self.gen
                        .as_mut()
                        .unwrap()
                        .invalidate_idf(&val.target().unwrap().idf().unwrap());
                }
            }
        }
        ast.to_ast()
    }

    pub fn visitIfStatement(&mut self, ast: &mut IfStatement) -> Statement
// """
    // Rule (6) + additional support for private conditions

    // If the condition is public, guard conditions are introduced for both branches if any of the branches contains private expressions.

    // If the condition is private, the whole if statement is inlined into the circuit. The only side-effects which are allowed
    // inside the branch bodies are assignment statements with an lhs@me. (anything else would leak private information).
    // The if statement will be replaced by an assignment statement where the lhs is a tuple of all locations which are written
    // in either branch and rhs is a tuple of the corresponding circuit outputs.
    // """
    {
        if ast.condition.annotated_type().unwrap().is_public() {
            if contains_private_expr(Some(ast.then_branch.to_ast()))
                || contains_private_expr(ast.else_branch.as_ref().map(|v| v.to_ast()))
            {
                let before_if_state = self.gen.as_ref().unwrap()._remapper.0.get_state();
                let guard_var = self
                    .gen
                    .as_mut()
                    .unwrap()
                    .add_to_circuit_inputs(&mut ast.condition);
                ast.condition = guard_var.get_loc_expr(Some(ast.to_ast())).expr().unwrap();
                self.gen.as_mut().unwrap().guarded(guard_var.clone(), true);
                {
                    ast.then_branch = self
                        .visit(Some(ast.then_branch.to_ast()))
                        .unwrap()
                        .block()
                        .unwrap();
                    self.gen
                        .as_mut()
                        .unwrap()
                        ._remapper
                        .0
                        .set_state(&before_if_state);
                }
                if ast.else_branch.is_some() {
                    self.gen.as_mut().unwrap().guarded(guard_var, false);

                    ast.else_branch = self
                        .visit(Some(ast.else_branch.as_ref().unwrap().to_ast()))
                        .unwrap()
                        .block();
                    self.gen
                        .as_mut()
                        .unwrap()
                        ._remapper
                        .0
                        .set_state(&before_if_state);
                }

                //Invalidate values modified in either branch
                for val in &ast.statement_base.ast_base.modified_values {
                    if val.key().is_none() {
                        self.gen
                            .as_mut()
                            .unwrap()
                            .invalidate_idf(&val.target().unwrap().idf().unwrap());
                    }
                }
            } else {
                ast.condition = self
                    .expr_trafo
                    .visit(Some(ast.condition.to_ast()))
                    .unwrap()
                    .expr()
                    .unwrap();
                ast.then_branch = self
                    .visit(Some(ast.then_branch.to_ast()))
                    .unwrap()
                    .block()
                    .unwrap();
                if ast.else_branch.is_some() {
                    ast.else_branch = self
                        .visit(Some(ast.else_branch.as_ref().unwrap().to_ast()))
                        .unwrap()
                        .block();
                }
            }
            (*ast).to_statement()
        } else {
            self.gen
                .as_mut()
                .unwrap()
                .evaluate_stmt_in_circuit(ast.to_statement())
                .to_statement()
        }
    }
    pub fn visitWhileStatement(&self, ast: WhileStatement) -> WhileStatement
//Loops must always be purely public
    {
        assert!(!contains_private_expr(Some(ast.condition.to_ast())));
        assert!(!contains_private_expr(Some(ast.body.to_ast())));
        ast
    }

    pub fn visitDoWhileStatement(&self, ast: DoWhileStatement) -> DoWhileStatement
//Loops must always be purely public
    {
        assert!(!contains_private_expr(Some(ast.condition.to_ast())));
        assert!(!contains_private_expr(Some(ast.body.to_ast())));
        ast
    }

    pub fn visitForStatement(&self, ast: &mut ForStatement) -> ForStatement {
        if ast.init.is_some()
        //Init is the only part of a for loop which may contain private expressions
        {
            ast.init = self
                .visit(Some(ast.init.as_ref().unwrap().to_ast()))
                .unwrap()
                .init()
                .map(|i| Box::new(i));
            ast.statement_base
                .pre_statements
                .extend(ast.init.as_ref().unwrap().pre_statements());
        }
        assert!(!contains_private_expr(Some(ast.condition.to_ast())));
        assert!(
            !ast.update.is_some()
                || !contains_private_expr(ast.update.as_ref().map(|v| v.to_ast()))
        );
        assert!(!contains_private_expr(Some(ast.body.to_ast()))); //OR fixed size loop -> static analysis can prove that loop terminates in fixed //iterations
        ast.clone()
    }

    pub fn visitContinueStatement(&self, ast: ContinueStatement) -> ContinueStatement {
        ast
    }

    pub fn visitBreakStatement(&self, ast: BreakStatement) -> BreakStatement {
        ast
    }

    pub fn visitReturnStatement(&mut self, ast: &mut ReturnStatement) -> Option<AST>
// """
    // Handle return statement.

    // If the function requires verification, the return statement is replaced by an assignment to a return variable.
    // (which will be returned at the very end of the function body, after any verification wrapper code).
    // Otherwise only the expression is transformed.
    // """
    {
        if ast
            .statement_base
            .function
            .as_ref()
            .unwrap()
            .requires_verification
        {
            if ast.expr.is_none() {
                return None;
            }
            assert!(!self.gen.as_ref().unwrap().has_return_var);
            self.gen.as_mut().unwrap().has_return_var = true;
            let expr = self
                .expr_trafo
                .visit(Some(ast.expr.as_ref().unwrap().to_ast()));
            let ret_args = ast
                .statement_base
                .function
                .as_ref()
                .unwrap()
                .return_var_decls
                .iter()
                .map(|vd| {
                    let mut idf = IdentifierExpr::new(
                        IdentifierExprUnion::Identifier(
                            *vd.identifier_declaration_base.idf.clone(),
                        ),
                        None,
                    );
                    idf.location_expr_base.target = Some(Box::new(vd.to_ast()));
                    idf.to_expr()
                })
                .collect();
            let mut te = TupleExpr::new(ret_args).assign(expr.unwrap().expr().unwrap());
            te.set_pre_statements(ast.statement_base.pre_statements.clone());
            Some(te.to_ast())
        } else {
            ast.expr = if let Some(AST::Expression(expr)) =
                self.expr_trafo.visit(ast.expr.clone().map(|e| e.to_ast()))
            {
                Some(expr)
            } else {
                None
            };
            Some(ast.to_ast())
        }
    }

    pub fn visitExpression(&self, ast: Expression)
    // """Fail if there are any untransformed expressions left."""
    {
        assert!(false, "Missed an expression of type {:?}", ast);
    }
}
// class ZkayExpressionTransformer(AstTransformerVisitor)
// """
// Roughly corresponds to T_L / T_e from paper.

// T_L and T_e are equivalent here, because parameter encryption checks are handled in the verification wrapper of the function body.
// In addition to the features described in the paper, this transformer also supports primitive type casting,
// tuples (multiple return values), operations with short-circuiting and function calls.
// """
#[derive(Clone)]
pub struct ZkayExpressionTransformer {
    gen: Option<Box<CircuitHelper>>,
}
impl TransformerVisitorEx for ZkayExpressionTransformer {}
impl AstTransformerVisitor for ZkayExpressionTransformer {
    fn default() -> Self {
        Self::new(None)
    }

    fn visit(&self, ast: Option<AST>) -> Option<AST> {
        // self._visit_internal(ast)
        None
    }
    fn visitBlock(
        &self,
        ast: Option<AST>,
        guard_cond: Option<HybridArgumentIdf>,
        guard_val: Option<bool>,
    ) -> Option<AST> {
        // self.visit_children(ast)
        None
    }
}
impl ZkayExpressionTransformer {
    pub fn new(current_generator: Option<Box<CircuitHelper>>) -> Self
// super().__init__()
        // self.gen.unwrap() = current_generator
    {
        Self {
            gen: current_generator,
        }
    }

    // @staticmethod
    pub fn visitMeExpr(ast: MeExpr) -> Expression
// """Replace me with msg.sender."""
    {
        replace_expr(
            &ast.to_expr(),
            &mut LocationExpr::IdentifierExpr(IdentifierExpr::new(
                IdentifierExprUnion::String(String::from("msg")),
                None,
            ))
            .to_expr(),
            false,
        )
        .to_location_expr()
        .unwrap()
        .dot(IdentifierExprUnion::String(String::from("sender")))
        .as_type(AST::AnnotatedTypeName(AnnotatedTypeName::address_all()))
        .to_expr()
    }

    pub fn visitLiteralExpr(&self, ast: LiteralExpr) -> LiteralExpr
// """Rule (7), don"t modify constants."""
    {
        ast
    }

    pub fn visitIdentifierExpr(&self, ast: IdentifierExpr) -> IdentifierExpr
// """Rule (8), don"t modify identifiers."""
    {
        ast
    }

    pub fn visitIndexExpr(&self, ast: IndexExpr) -> Expression
// """Rule (9), transform location and index expressions separately."""
    {
        replace_expr(
            &ast.to_expr(),
            &mut self
                .visit(ast.arr.map(|a| (*a).to_ast()))
                .unwrap()
                .to_location_expr()
                .unwrap()
                .index(ExprUnion::Expression(
                    self.visit(Some((*ast.key).to_ast()))
                        .unwrap()
                        .expr()
                        .unwrap(),
                ))
                .to_expr(),
            false,
        )
    }

    pub fn visitMemberAccessExpr(&self, ast: MemberAccessExpr) {
        self.visit_children(Some(ast.to_ast()));
    }

    pub fn visitTupleExpr(&self, ast: TupleExpr) {
        self.visit_children(Some(ast.to_ast()));
    }

    pub fn visitReclassifyExpr(&mut self, ast: ReclassifyExpr) -> LocationExpr
// """
    // Rule (11), trigger a boundary crossing.

    // The reclassified expression is evaluated in the circuit and its result is made available in solidity.
    // """
    {
        self.gen.as_mut().unwrap().evaluate_expr_in_circuit(
            &mut ast.expr().unwrap(),
            &ast.privacy()
                .unwrap()
                .privacy_annotation_label()
                .unwrap()
                .into(),
            &ast.annotated_type().unwrap().homomorphism,
        )
    }

    pub fn visitBuiltinFunction(&self, ast: BuiltinFunction) -> BuiltinFunction {
        ast
    }

    pub fn visitFunctionCallExpr(&mut self, mut ast: FunctionCallExpr) -> AST {
        if is_instance(&ast.func().unwrap(), ASTType::BuiltinFunction) {
            if ast.func().unwrap().is_private()
            // """
            // Modified Rule (12) builtin functions with private operands and homomorphic operations on ciphertexts
            // are evaluated inside the circuit.

            // A private expression on its own (like an IdentifierExpr referring to a private variable) is not enough to trigger a
            // boundary crossing (assignment of private variables is a public operation).
            // """
            {
                let privacy_label = ast
                    .annotated_type()
                    .unwrap()
                    .privacy_annotation
                    .unwrap()
                    .privacy_annotation_label();
                return self
                    .gen
                    .as_mut()
                    .unwrap()
                    .evaluate_expr_in_circuit(
                        &mut ast.to_expr(),
                        &(privacy_label.unwrap().into()),
                        &(ast.func().unwrap().homomorphism().into()),
                    )
                    .to_ast();
            } else
            // """
            // Rule (10) with additional short-circuit handling.

            // Builtin operations on public operands are normally left untransformed, but if the builtin function has
            // short-circuiting semantics, guard conditions must be added if any of the public operands contains
            // nested private expressions.
            // """
            //handle short-circuiting
            {
                let mut args = ast.args();
                if ast.func().unwrap().has_shortcircuiting()
                    && args[1..]
                        .iter()
                        .any(|arg| contains_private_expr(Some(arg.to_ast())))
                {
                    let op = &ast.func().unwrap().op().unwrap();
                    let guard_var = self
                        .gen
                        .as_mut()
                        .unwrap()
                        .add_to_circuit_inputs(&mut args[0]);
                    args[0] = guard_var.get_loc_expr(Some(ast.to_ast())).expr().unwrap();
                    if op == "ite" {
                        args[1] = self
                            .visit_guarded_expression(guard_var.clone(), true, &mut args[1].clone())
                            .expr()
                            .unwrap();
                        args[2] = self
                            .visit_guarded_expression(
                                guard_var.clone(),
                                false,
                                &mut args[2].clone(),
                            )
                            .expr()
                            .unwrap();
                    } else if op == "||" {
                        args[1] = self
                            .visit_guarded_expression(
                                guard_var.clone(),
                                false,
                                &mut args[1].clone(),
                            )
                            .expr()
                            .unwrap();
                    } else if op == "&&" {
                        args[1] = self
                            .visit_guarded_expression(guard_var.clone(), true, &mut args[1].clone())
                            .expr()
                            .unwrap();
                    }
                    ast.set_args(args);
                }

                return self.visit_children(Some(ast.to_ast())).unwrap();
            }
        } else if ast.is_cast()
        // """Casts are handled either in public or inside the circuit depending on the privacy of the casted expression."""
        {
            assert!(is_instance(
                &ast.func().unwrap().target().map(|t| *t).unwrap(),
                ASTType::EnumDefinition
            ));
            if ast.args()[0].evaluate_privately() {
                let privacy_label = ast
                    .annotated_type()
                    .unwrap()
                    .privacy_annotation
                    .unwrap()
                    .privacy_annotation_label();
                return self
                    .gen
                    .as_mut()
                    .unwrap()
                    .evaluate_expr_in_circuit(
                        &mut ast.to_expr(),
                        &privacy_label.unwrap().into(),
                        &ast.annotated_type().unwrap().homomorphism,
                    )
                    .to_ast();
            } else {
                return self.visit_children(Some(ast.to_ast())).unwrap();
            }
        } else
        // """
        // Handle normal function calls (outside private expression case).

        // The called functions are allowed to have side effects,
        // if the function does not require verification it can even be recursive.
        // """
        {
            assert!(is_instance(&ast.func().unwrap(), ASTType::LocationExprBase));
            let mut ast = self.visit_children(Some(ast.to_ast())).unwrap();
            if ast
                .func()
                .unwrap()
                .target()
                .unwrap()
                .constructor_or_function_definition()
                .unwrap()
                .requires_verification_when_external
            //Reroute the function call to the corresponding internal function if the called function was split into external/internal.
            {
                if !is_instance(&ast.func().unwrap(), ASTType::IdentifierExpr) {
                    unimplemented!();
                }
                ast.set_func_idf_name(
                    CFG.lock()
                        .unwrap()
                        .get_internal_name(&*ast.func().unwrap().target().unwrap()),
                );
            }

            if ast
                .func()
                .unwrap()
                .target()
                .unwrap()
                .requires_verification()
            //If the target function has an associated circuit, make this function"s circuit aware of the call.
            {
                let cf = if let AST::Expression(Expression::FunctionCallExpr(fce)) = &ast {
                    Some(fce.clone())
                } else {
                    None
                };
                self.gen.as_mut().unwrap().call_function(&cf.unwrap());
            } else if ast
                .func()
                .unwrap()
                .target()
                .unwrap()
                .constructor_or_function_definition()
                .unwrap()
                .has_side_effects()
                && self.gen.is_some()
            //Invalidate modified state variables for the current circuit
            {
                for val in &ast.ast_base().unwrap().modified_values {
                    if val.key().is_none()
                        && is_instance(
                            &val.target().map(|t| *t).unwrap(),
                            ASTType::StateVariableDeclaration,
                        )
                    {
                        self.gen
                            .as_mut()
                            .unwrap()
                            .invalidate_idf(&val.target().unwrap().idf().unwrap());
                    }
                }
            }

            //The call will be present as a normal function call in the output solidity code.
            ast.to_ast()
        }
    }
    pub fn visit_guarded_expression(
        &mut self,
        guard_var: HybridArgumentIdf,
        if_true: bool,
        expr: &mut Expression,
    ) -> AST {
        let prelen = expr.statement().unwrap().pre_statements().len();

        //Transform expression with guard condition in effect
        self.gen
            .as_mut()
            .unwrap()
            .guarded(guard_var.clone(), if_true);
        let ret = self.visit(Some(expr.to_ast()));

        //If new pre statements were added, they must be guarded using an if statement in the public solidity code
        let new_pre_stmts = expr.statement().unwrap().pre_statements()[prelen..].to_vec();
        if !new_pre_stmts.is_empty() {
            let mut cond_expr: AST = guard_var.get_loc_expr(None).into();
            if let AST::Expression(Expression::LiteralExpr(LiteralExpr::BooleanLiteralExpr(
                ref mut cond_expr,
            ))) = cond_expr
            {
                *cond_expr = BooleanLiteralExpr::new(cond_expr.value == if_true);
            } else if !if_true {
                cond_expr = cond_expr.expr().unwrap().unop(String::from("!")).to_ast();
            }
            expr.set_statement_pre_statements(
                expr.statement().unwrap().pre_statements()[..prelen]
                    .iter()
                    .cloned()
                    .chain([IfStatement::new(
                        cond_expr.expr().unwrap(),
                        Block::new(new_pre_stmts, false),
                        None,
                    )
                    .to_ast()])
                    .collect(),
            );
        }
        ret.unwrap()
    }

    pub fn visitPrimitiveCastExpr(&mut self, ast: PrimitiveCastExpr) -> AST
// """Casts are handled either in public or inside the circuit depending on the privacy of the casted expression."""
    {
        if ast.expression_base.evaluate_privately {
            let privacy_label = ast
                .expression_base
                .annotated_type
                .as_ref()
                .unwrap()
                .privacy_annotation
                .as_ref()
                .unwrap()
                .privacy_annotation_label();
            self.gen
                .as_mut()
                .unwrap()
                .evaluate_expr_in_circuit(
                    &mut ast.to_expr(),
                    &(privacy_label.unwrap().into()),
                    &ast.expression_base
                        .annotated_type
                        .as_ref()
                        .unwrap()
                        .homomorphism,
                )
                .to_ast()
        } else {
            self.visit_children(Some(ast.to_ast())).unwrap()
        }
    }

    pub fn visitExpression(&self, ast: Expression) {
        // raise NotImplementedError()
        unimplemented!();
    }
}
// class ZkayCircuitTransformer(AstTransformerVisitor)
// """
// Corresponds to T_phi from paper.

// This extends the abstract circuit representation while transforming private expressions and statements.
// Private expressions can never have side effects.
// Private statements may contain assignment statements with lhs@me (no other types of side effects are allowed).
// """
#[derive(Clone)]
pub struct ZkayCircuitTransformer {
    gen: Option<Box<CircuitHelper>>,
}

impl TransformerVisitorEx for ZkayCircuitTransformer {}
impl AstTransformerVisitor for ZkayCircuitTransformer {
    fn default() -> Self {
        Self::new(None)
    }

    fn visit(&self, ast: Option<AST>) -> Option<AST> {
        // self._visit_internal(ast)
        None
    }
    fn visitBlock(
        &self,
        ast: Option<AST>,
        guard_cond: Option<HybridArgumentIdf>,
        guard_val: Option<bool>,
    ) -> Option<AST> {
        // self.visit_children(ast)
        None
    }
}
impl ZkayCircuitTransformer {
    pub fn new(current_generator: Option<Box<CircuitHelper>>) -> Self {
        Self {
            gen: current_generator,
        }
    }
    // super().__init__()
    // self.gen.unwrap() = current_generator

    pub fn visitLiteralExpr(&self, ast: LiteralExpr) -> LiteralExpr
// """Rule (13), don"t modify constants."""
    {
        ast
    }

    pub fn visitIndexExpr(&mut self, ast: IndexExpr) -> LocationExpr {
        self.transform_location(LocationExpr::IndexExpr(ast))
    }

    pub fn visitIdentifierExpr(&mut self, mut ast: IdentifierExpr) -> LocationExpr {
        if !is_instance(&*ast.idf, ASTType::HybridArgumentIdf)
        //If ast is not already transformed, get current SSA version
        {
            ast = self.gen.as_ref().unwrap().get_remapped_idf_expr(ast);
        }
        if is_instance(&ast, ASTType::IdentifierExpr)
            && is_instance(&*ast.idf, ASTType::HybridArgumentIdf)
        //The current version of ast.idf is already in the circuit
        {
            assert!(ast.idf.arg_type() != HybridArgType::PubContractVal);
            LocationExpr::IdentifierExpr(ast)
        } else
        //ast is not yet in the circuit -> move it in
        {
            self.transform_location(LocationExpr::IdentifierExpr(ast))
        }
    }

    pub fn transform_location(&mut self, loc: LocationExpr) -> LocationExpr
// """Rule (14), move location into the circuit."""
    {
        LocationExpr::IdentifierExpr(
            self.gen
                .as_mut()
                .unwrap()
                .add_to_circuit_inputs(&mut loc.to_expr())
                .get_idf_expr(&None),
        )
    }

    pub fn visitReclassifyExpr(&mut self, ast: ReclassifyExpr) -> AST
// """Rule (15), boundary crossing if analysis determined that it is """
    {
        if ast.annotated_type().unwrap().is_cipher()
        //We need a homomorphic ciphertext -> make sure the correct encryption of the value is available
        {
            let orig_type = ast.annotated_type().unwrap().zkay_type();
            let orig_privacy = orig_type
                .privacy_annotation
                .unwrap()
                .privacy_annotation_label();
            let orig_homomorphism = orig_type.homomorphism;
            self.gen
                .as_mut()
                .unwrap()
                .evaluate_expr_in_circuit(
                    &mut ast.expr().unwrap(),
                    &orig_privacy.unwrap().into(),
                    &orig_homomorphism,
                )
                .to_ast()
        } else if ast.expr().unwrap().evaluate_privately() {
            self.visit(Some(ast.expr().unwrap().to_ast())).unwrap()
        } else {
            assert!(ast.expr().unwrap().annotated_type().unwrap().is_public());
            self.gen
                .as_mut()
                .unwrap()
                .add_to_circuit_inputs(&mut ast.expr().unwrap())
                .get_idf_expr(&None)
                .to_ast()
        }
    }

    pub fn visitExpression(&self, ast: Expression) -> AST
// """Rule (16), other expressions don"t need special treatment."""
    {
        self.visit_children(Some(ast.to_ast())).unwrap()
    }

    pub fn visitFunctionCallExpr(&mut self, mut ast: FunctionCallExpr) -> Expression {
        let t = ast.annotated_type().unwrap().type_name;

        //Constant folding for literal types
        if let TypeName::ElementaryTypeName(ElementaryTypeName::BooleanLiteralType(t)) = *t {
            return replace_expr(
                &ast.to_expr(),
                &mut BooleanLiteralExpr::new(t.value()).to_expr(),
                false,
            );
        } else if let TypeName::ElementaryTypeName(ElementaryTypeName::NumberTypeName(
            NumberTypeName::NumberLiteralType(t),
        )) = *t
        {
            return replace_expr(
                &ast.to_expr(),
                &mut NumberLiteralExpr::new(t.value(), false).to_expr(),
                false,
            );
        }

        if is_instance(&ast.func().unwrap(), ASTType::BuiltinFunction) {
            if ast.func().unwrap().homomorphism() != Homomorphism::non_homomorphic()
            //To perform homomorphic operations, we require the recipient"s public key
            {
                let crypto_params = CFG
                    .lock()
                    .unwrap()
                    .user_config
                    .get_crypto_params(&ast.func().unwrap().homomorphism());
                let recipient = ast
                    .annotated_type()
                    .unwrap()
                    .zkay_type()
                    .privacy_annotation
                    .unwrap()
                    .privacy_annotation_label();
                let mut s = ast.statement().unwrap();
                ast.set_public_key(Some(Box::new(
                    self.gen.as_mut().unwrap()._require_public_key_for_label_at(
                        Some(&mut s),
                        &recipient.unwrap().into(),
                        &crypto_params,
                    ),
                )));

                if &ast.func().unwrap().op().unwrap() == "*"
                //special case: private scalar multiplication using additive homomorphism
                //TODO ugly hack below removes ReclassifyExpr
                {
                    let mut new_args = vec![];
                    for mut arg in ast.args() {
                        if is_instance(&arg, ASTType::ReclassifyExpr) {
                            arg = arg.expr().unwrap();
                            ast.set_func_rerand_using(Some(Box::new(
                                self.gen
                                    .as_mut()
                                    .unwrap()
                                    .get_randomness_for_rerand(ast.to_expr()),
                            )));
                        //result requires re-randomization
                        } else if arg.annotated_type().unwrap().is_private() {
                            arg.set_annotated_type(AnnotatedTypeName::cipher_type(
                                arg.annotated_type().unwrap(),
                                Some(ast.func().unwrap().homomorphism()),
                            ));
                        }
                        new_args.push(arg);
                    }
                    ast.set_args(new_args);
                } else
                //We require all non-public arguments to be present as ciphertexts
                {
                    for arg in ast.args().iter_mut() {
                        if arg.annotated_type().unwrap().is_private() {
                            arg.set_annotated_type(AnnotatedTypeName::cipher_type(
                                arg.annotated_type().unwrap(),
                                Some(ast.func().unwrap().homomorphism()),
                            ));
                        }
                    }
                }
            }

            //Builtin functions are supported natively by the circuit
            return self
                .visit_children(Some(ast.to_ast()))
                .unwrap()
                .expr()
                .unwrap();
        }

        let fdef = &*ast.func().unwrap().target().unwrap();
        assert!(fdef
            .constructor_or_function_definition()
            .unwrap()
            .is_function());
        assert!(!fdef
            .constructor_or_function_definition()
            .unwrap()
            .return_parameters
            .is_empty());
        assert!(
            fdef.constructor_or_function_definition()
                .unwrap()
                .has_static_body
        );

        //Function call inside private expression -> entire body will be inlined into circuit.
        //Function must not have side-effects (only pure and view is allowed) and cannot have a nonstatic body (i.e. recursion)
        let mut fce = if let FunctionCallExpr::FunctionCallExpr(ref mut fce) = ast {
            Some(fce.clone())
        } else {
            None
        };
        return self
            .gen
            .as_mut()
            .unwrap()
            .inline_function_call_into_circuit(&mut fce.unwrap())
            .unwrap()
            .expr()
            .unwrap();
    }

    pub fn visitReturnStatement(&mut self, ast: &mut ReturnStatement) {
        self.gen.as_mut().unwrap().add_return_stmt_to_circuit(ast)
    }

    pub fn visitAssignmentStatement(&mut self, ast: &mut AssignmentStatement) {
        self.gen.as_mut().unwrap().add_assignment_to_circuit(ast)
    }

    pub fn visitVariableDeclarationStatement(&mut self, ast: &mut VariableDeclarationStatement) {
        self.gen.as_mut().unwrap().add_var_decl_to_circuit(ast)
    }

    pub fn visitIfStatement(&mut self, ast: &mut IfStatement) {
        self.gen.as_mut().unwrap().add_if_statement_to_circuit(ast)
    }

    pub fn visitBlock(
        &mut self,
        ast: &mut Block,
        guard_cond: Option<HybridArgumentIdf>,
        guard_val: Option<bool>,
    ) {
        self.gen
            .as_mut()
            .unwrap()
            .add_block_to_circuit(ast, guard_cond, guard_val)
    }

    pub fn visitStatement(&mut self, ast: Statement)
    // """Fail if statement type was not handled."""
    // raise NotImplementedError("Unsupported statement")
    {
        unimplemented!("Unsupported statement")
    }
}