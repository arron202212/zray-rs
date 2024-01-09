use crate::config::CFG;
// use crate::type_check::type_exceptions::TypeException
use crate::zkay_ast::ast::{
    AssignmentStatement, BooleanLiteralType, BuiltinFunction, ConstructorOrFunctionDefinition,
    Expression, FunctionCallExpr, FunctionTypeName, IfStatement, IndexExpr, LocationExpr,
    NumberLiteralType, PrimitiveCastExpr, ReclassifyExpr, ReturnStatement, Statement,
    StatementList, AST,
};
use crate::zkay_ast::visitor::function_visitor::FunctionVisitor;

pub fn check_circuit_compliance(ast: AST) {
    // """
    // determines for every function whether it can be used inside a circuit
    // """
    let v = DirectCanBePrivateDetector::new();
    v.visit(ast);

    let v = IndirectCanBePrivateDetector::new();
    v.visit(ast);

    let v = CircuitComplianceChecker::new();
    v.visit(ast);

    check_for_nonstatic_function_calls_or_not_circuit_inlineable_in_private_exprs(ast)
}
// class DirectCanBePrivateDetector(FunctionVisitor)
pub struct DirectCanBePrivateDetector;
impl DirectCanBePrivateDetector {
    pub fn visitFunctionCallExpr(self, ast: FunctionCallExpr) {
        if isinstance(ast.func, BuiltinFunction) {
            if !ast.func.is_private {
                can_be_private = ast.func.can_be_private();
                if ast.func.is_eq() || ast.func.is_ite() {
                    can_be_private &= ast.args[1].annotated_type.type_name.can_be_private();
                }
                ast.statement.function.can_be_private &= can_be_private;
                //TODO to relax this for public expressions,
                // public identifiers must use SSA remapped values (since the function is inlined)
            }
        }
        for arg in ast.args {
            self.visit(arg);
        }
    }

    pub fn visitLocationExpr(self, ast: LocationExpr) {
        t = ast.annotated_type.type_name;
        ast.statement.function.can_be_private &= t.can_be_private();
        self.visitChildren(ast);
    }

    pub fn visitReclassifyExpr(self, ast: ReclassifyExpr) {
        return self.visit(ast.expr);
    }

    pub fn visitAssignmentStatement(self, ast: AssignmentStatement) {
        self.visitChildren(ast)
    }

    pub fn visitVariableDeclarationStatement(self, ast: AssignmentStatement) {
        self.visitChildren(ast)
    }

    pub fn visitReturnStatement(self, ast: ReturnStatement) {
        self.visitChildren(ast)
    }

    pub fn visitIfStatement(self, ast: IfStatement) {
        self.visitChildren(ast)
    }

    pub fn visitStatementList(self, ast: StatementList) {
        self.visitChildren(ast)
    }

    pub fn visitStatement(self, ast: Statement) {
        //All other statement types are not supported inside circuit (for now)
        ast.function.can_be_private = False;
    }
}
// class IndirectCanBePrivateDetector(FunctionVisitor)
pub struct IndirectCanBePrivateDetector;
impl IndirectCanBePrivateDetector {
    pub fn visitConstructorOrFunctionDefinition(self, ast: ConstructorOrFunctionDefinition) {
        if ast.can_be_private {
            for fct in ast.called_functions {
                if !fct.can_be_private {
                    ast.can_be_private = False;
                    return;
                }
            }
        }
    }
}
// class CircuitComplianceChecker(FunctionVisitor)
pub struct CircuitComplianceChecker {
    priv_setter: PrivateSetter,
    inside_privif_stmt: bool,
}
impl CircuitComplianceChecker {
    // pub fn __init__(self)
    //     super().__init__()
    //     self.priv_setter = PrivateSetter()
    //     self.inside_privif_stmt = False
    pub fn new() -> Self {
        Self {
            priv_setter: PrivateSetter::new(),
            inside_privif_stmt: false,
        }
    }
    // @staticmethod
    pub fn should_evaluate_public_expr_in_circuit(expr: Expression) -> bool {
        assert!(expr.annotated_type.is_some());
        if cfg.opt_eval_constexpr_in_circuit {
            if isinstance(
                expr.annotated_type.type_name,
                (NumberLiteralType, BooleanLiteralType),
            ) {
                //Expressions for which the value is known at compile time -> embed constant expression value into the circuit
                return True;
            }

            if isinstance(expr, PrimitiveCastExpr)
                && isinstance(
                    expr.expr.annotated_type.type_name,
                    (NumberLiteralType, BooleanLiteralType),
                )
            {
                //Constant casts should also be evaluated inside the circuit
                return True;
            }
        }

        // try
        check_for_nonstatic_function_calls_or_not_circuit_inlineable_in_private_exprs(expr);
        // except TypeException
        //     //Cannot evaluate inside circuit -> never do it
        //     return False

        //Could evaluate in circuit, use analysis to determine whether this would be better performance wise
        //(If this avoids unnecessary encryption operations it may be cheaper)
        return False;
    }

    pub fn visitIndexExpr(self, ast: IndexExpr) {
        if ast.evaluate_privately {
            assert!(ast.key.annotated_type.is_public());
            self.priv_setter.set_evaluation(ast.key, False);
        }
        return self.visitChildren(ast);
    }

    pub fn visitReclassifyExpr(self, ast: ReclassifyExpr) {
        if self.inside_privif_stmt
            && !ast
                .statement
                .before_analysis
                .same_partition(ast.privacy.privacy_annotation_label(), Expression.me_expr())
        {
            assert!(false,"Revealing information to other parties is not allowed inside private if statements", ast)
        }
        if ast.expr.annotated_type.is_public() {
            eval_in_public = False;
            // try
            self.priv_setter
                .set_evaluation(ast, evaluate_privately = True);
            // except TypeException
            //     eval_in_public = True
            if eval_in_public || !self.should_evaluate_public_expr_in_circuit(ast.expr) {
                self.priv_setter
                    .set_evaluation(ast.expr, evaluate_privately = False);
            }
        } else {
            self.priv_setter
                .set_evaluation(ast, evaluate_privately = True);
        }
        self.visit(ast.expr);
    }

    pub fn visitFunctionCallExpr(self, ast: FunctionCallExpr) {
        if isinstance(ast.func, BuiltinFunction) && ast.func.is_private {
            self.priv_setter
                .set_evaluation(ast, evaluate_privately = True);
        } else if ast.is_cast && ast.annotated_type.is_private() {
            self.priv_setter
                .set_evaluation(ast, evaluate_privately = True);
        }
        self.visitChildren(ast);
    }

    pub fn visitPrimitiveCastExpr(self, ast: PrimitiveCastExpr) {
        if ast.expr.annotated_type.is_private() {
            self.priv_setter
                .set_evaluation(ast, evaluate_privately = True);
        }
        self.visitChildren(ast);
    }

    pub fn visitIfStatement(self, ast: IfStatement) {
        old_in_privif_stmt = self.inside_privif_stmt;
        if ast.condition.annotated_type.is_private() {
            mod_vals = set(ast.then_branch.modified_values.keys());
            if ast.else_branch.is_some() {
                mod_vals = mod_vals.union(ast.else_branch.modified_values);
            }
            for val in mod_vals {
                if !val
                    .target
                    .annotated_type
                    .zkay_type
                    .type_name
                    .is_primitive_type()
                {
                    assert!(false,"Writes to non-primitive type variables are not allowed inside private if statements", ast)
                }
                if val.in_scope_at(ast)
                    && !ast
                        .before_analysis
                        .same_partition(val.privacy, Expression.me_expr())
                {
                    assert!(false,"If statement with private condition must not contain side effects to variables with owner != me", ast)
                }
            }
            self.inside_privif_stmt = True;
            self.priv_setter
                .set_evaluation(ast, evaluate_privately = True);
        }
        self.visitChildren(ast);
        self.inside_privif_stmt = old_in_privif_stmt;
    }
}
// class PrivateSetter(FunctionVisitor)
pub struct PrivateSetter {
    evaluate_privately: Option<bool>,
}
impl PrivateSetter {
    // pub fn __init__(self)
    //     super().__init__()
    //     self.evaluate_privately = None
    pub fn new() -> Self {
        Self {
            evaluate_privately: None,
        }
    }
    pub fn set_evaluation(self, ast: vec![Expression, Statement], evaluate_privately: bool) {
        self.evaluate_privately = evaluate_privately;
        self.visit(ast);
        self.evaluate_privately = None;
    }

    pub fn visitFunctionCallExpr(self, ast: FunctionCallExpr) {
        if self.evaluate_privately
            && isinstance(ast.func, LocationExpr)
            && !ast.is_cast
            && ast.func.target.has_side_effects
        {
            assert!(
                false,
                "Expressions with side effects are not allowed inside private expressions",
                ast
            )
        }
        self.visitExpression(ast);
    }

    pub fn visitExpression(self, ast: Expression) {
        assert!(self.evaluate_privately.is_some());
        ast.evaluate_privately = self.evaluate_privately;
        self.visitChildren(ast);
    }
}
pub fn check_for_nonstatic_function_calls_or_not_circuit_inlineable_in_private_exprs(ast: AST) {
    NonstaticOrIncompatibilityDetector().visit(ast)
}

// class NonstaticOrIncompatibilityDetector(FunctionVisitor)
pub struct NonstaticOrIncompatibilityDetector;
impl NonstaticOrIncompatibilityDetector {
    pub fn visitFunctionCallExpr(self, ast: FunctionCallExpr) {
        can_be_private = True;
        has_nonstatic_call = False;
        if ast.evaluate_privately && !ast.is_cast {
            if isinstance(ast.func, LocationExpr) {
                assert!(ast.func.target.is_some());
                assert!(isinstance(
                    ast.func.target.annotated_type.type_name,
                    FunctionTypeName
                ));
                has_nonstatic_call |= !ast.func.target.has_static_body;
                can_be_private &= ast.func.target.can_be_private;
            } else if isinstance(ast.func, BuiltinFunction) {
                can_be_private &=
                    (ast.func.can_be_private() || ast.annotated_type.type_name.is_literal);
                if ast.func.is_eq() || ast.func.is_ite() {
                    can_be_private &= ast.args[1].annotated_type.type_name.can_be_private();
                }
            }
        }
        if has_nonstatic_call {
            assert!(
                false,
                "Function calls to non static functions are not allowed inside private expressions",
                ast
            )
        }
        if !can_be_private {
            assert!(false,
                "Calls to functions with operations which cannot be expressed as a circuit are not allowed inside private expressions", ast)
        }
        self.visitChildren(ast);
    }
}
