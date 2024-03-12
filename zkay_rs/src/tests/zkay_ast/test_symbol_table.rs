
// from zkay.examples.examples::simple, simple_storage, all_examples
// from zkay.tests.utils.test_examples::TestExamples
// from zkay.tests.zkay_unit_test::ZkayTestCase
// from zkay.zkay_ast.ast::SourceUnit, VariableDeclarationStatement, IdentifierExpr, \
//     AssignmentStatement
// from zkay.zkay_ast.build_ast::build_ast
// from zkay.zkay_ast.pointers.parent_setter::set_parents
// from zkay.zkay_ast.pointers.symbol_table::fill_symbol_table, link_identifiers, get_builtin_globals


// class TestSimpleAST(ZkayTestCase):

//     def get_ast_elements(self, ast: SourceUnit):
//         self.contract = ast.contracts[0]
//         self.f = self.contract.function_definitions[0]
//         self.body = self.f.body
//         self.decl_statement = self.body.statements[0]
//         assert (isinstance(self.decl_statement, VariableDeclarationStatement))
//         self.decl = self.decl_statement.variable_declaration
//         self.assignment = self.body.statements[1]
//         assert (isinstance(self.assignment, AssignmentStatement))
//         self.identifier_expr = self.assignment.lhs
//         assert (isinstance(self.identifier_expr, IdentifierExpr))

//     def test_fill_symbol_table(self):
//         ast = build_ast(simple.code())
//         fill_symbol_table(ast)

//         self.get_ast_elements(ast)

//         s = get_builtin_globals()
//         s.update({'Simple': self.contract.idf})
//         self.assertDictEqual(ast.names, s)
//         self.assertDictEqual(self.contract.names, {'f': self.f.idf})
//         self.assertDictEqual(self.body.names, {'x': self.decl.idf})

//     def test_link_identifiers(self):
//         ast = build_ast(simple.code())
//         set_parents(ast)
//         link_identifiers(ast)

//         self.get_ast_elements(ast)

//         self.assertEqual(self.identifier_expr.target, self.decl)
//         self.assertEqual(self.identifier_expr.annotated_type(), self.decl.annotated_type)


// class TestSimpleStorageAST(ZkayTestCase):

//     def test_fill_symbol_table(self):
//         ast = build_ast(simple_storage.code())
//         fill_symbol_table(ast)

//         contract = ast.contracts[0]

//         s = get_builtin_globals()
//         s.update({'SimpleStorage': contract.idf})
//         self.assertDictEqual(ast.names, s)

//     def test_link_identifiers(self):
//         ast = build_ast(simple_storage.code())
//         set_parents(ast)
//         link_identifiers(ast)
//         assignment = ast['SimpleStorage']['set'].body[0]
//         self.assertIsInstance(assignment, AssignmentStatement)

//         stored_data = assignment.lhs.target
//         self.assertEqual(stored_data, ast['SimpleStorage']['storedData'])

//         x = assignment.rhs.target
//         self.assertEqual(x.idf.name, 'x')


// @parameterized_class(('name', 'example'), all_examples)
// class TestSymbolTable(TestExamples):

//     def test_symbol_table(self):
//         ast = build_ast(self.example.code())
//         set_parents(ast)
//         fill_symbol_table(ast)
//         link_identifiers(ast)
//         contract = ast.contracts[0]
//         self.assertEqual(contract.idf.name, self.name)
