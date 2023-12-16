use antlr_rust::common_token_stream::CommonTokenStream;
use antlr_rust::token::{Token, TOKEN_EOF, TOKEN_HIDDEN_CHANNEL};
use antlr_rust::tree::{ParseTreeVisitorCompat, TerminalNode, Visitable};
// use crate::tree::{ErrorNode, Listenable, ParseTreeListener, TerminalNode};
// use antlr_rust::TokenSource;
// use  crate::solidity_parser::generated::solidityvisitor::{SolidityVisitor};
// use  crate::solidity_parser::parse::get_parse_tree;
use crate::solidity_parser::generated::solidityparser::SolidityParserContextType;
//  use antlr_rust::parser_rule_context::ParserRuleContext;
use antlr_rust::parser::ParserNodeType;
use antlr_rust::token_stream::TokenStream;
use antlr_rust::tree::ParseTree;
// use std::borrow::Borrow;
// use std::borrow::Cow;
use crate::solidity_parser::generated::soliditylexer::SolidityLexer;
use crate::solidity_parser::parse::MyErrorListener;
use antlr_rust::char_stream::InputData;
// use antlr_rust::token::GenericToken;
use antlr_rust::InputStream;

pub struct Emitter {
    code: Option<String>,
    next_token_index: i32,
    emitted: String,
}
impl<'input> Emitter {
    pub fn new(code: Option<String>) -> Self {
        Self {
            code,
            next_token_index: 0,
            emitted: String::new(),
        }
    }

    fn get_hidden_up_to(
        &mut self,
        node: &TerminalNode<'input, SolidityParserContextType>,
    ) -> String {
        // handle unavailable token stream by using spaces
        if self.code.is_none() {
            if self.next_token_index == 0 {
                self.next_token_index += 1;
            }
            return String::new();
        }

        // when token stream available: add hidden tokens
        let mut ret = String::new();

        let token_index = node.get_source_interval().a;
        let codes = if let Some(c) = &self.code {
            c.to_string()
        } else {
            String::new()
        };
        let c = codes.clone();
        let mut lexer = SolidityLexer::new(InputStream::new(c.as_str()));
        lexer.add_error_listener(Box::new(MyErrorListener { code: codes }));
        let token_stream = CommonTokenStream::new(lexer);
        while self.next_token_index <= token_index as i32 {
            let before = token_stream.get(self.next_token_index as isize);
            if before.get_channel() == TOKEN_HIDDEN_CHANNEL {
                ret += &before.get_text().to_display();
            }
            self.next_token_index += 1;
        }

        ret
    }
}
use crate::solidity_parser::generated::solidityvisitor::SolidityVisitorCompat;
impl<'input> SolidityVisitorCompat<'input> for Emitter {}

impl<'input> ParseTreeVisitorCompat<'input> for Emitter {
    type Node = SolidityParserContextType;
    type Return = String;
    fn temp_result(&mut self) -> &mut <Self as ParseTreeVisitorCompat<'input>>::Return {
        todo!()
    }
    fn visit_terminal(&mut self, node: &TerminalNode<'input, Self::Node>) -> Self::Return {
        let hidden = self.get_hidden_up_to(node);
        let code = if node.symbol.get_token_type() == TOKEN_EOF {
            String::new()
        } else {
            node.get_text()
        };

        self.emitted += &(hidden.to_owned() + code.as_str());
        self.emitted.clone()
    }

    fn visit_children(
        &mut self,
        node: &<SolidityParserContextType as ParserNodeType>::Type,
    ) -> Self::Return {
        for c in node.get_children() {
            c.accept(self)
        }
        self.emitted.clone()
    }
}

// pub fn normalize_code(code:&str)->Option<String>{
//     let tree = get_parse_tree(code);
//     let emitter = Emitter::new(None);
//     Some(emitter.visit(tree))
// }
