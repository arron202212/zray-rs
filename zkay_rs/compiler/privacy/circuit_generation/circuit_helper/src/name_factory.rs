use rccell::RcCell;
use zkay_ast::ast::{
    ASTFlatten, ASTInstanceOf, HybridArgType, HybridArgumentIdf, Identifier, TypeName,
};
#[derive(Clone)]
pub struct BaseNameFactory {
    pub base_name: String,
    pub count: RcCell<i32>,
}
// class BaseNameFactory:
// """A Base name factory can generate fresh, unused name strings with a given prefix"""
impl BaseNameFactory {
    pub fn new(base_name: String) -> Self {
        //  println!("=====BaseNameFactory==before=={}=", line!());
        Self {
            base_name,
            count: RcCell::new(0),
        }
    }
    // """
    // Generate a fresh name for a value of type t.

    // :param t: transformed type
    // :param inc: if true, the internal counter, which is used as part of fresh ids, is incremented
    // """
    pub fn get_new_name(&self, t: &RcCell<TypeName>, inc: bool) -> String {
        let postfix = match t.borrow() {
            _t if _t.is_key() => "key",
            _t if _t.is_cipher() => "cipher",
            _t if _t.is_randomness() => "rnd",
            _ => "plain",
        };
        let name = format!("{}{}_{postfix}", self.base_name, self.count.borrow());
        if inc {
            *self.count.borrow_mut() += 1;
        }
        name
    }
}

// class NameFactory(BaseNameFactory):
// """A Name factory can generate fresh, unused HybridArgumentIdfs with a given prefix."""
#[derive(Clone)]
pub struct NameFactory {
    pub base_name_factory: BaseNameFactory,
    pub arg_type: HybridArgType,
    pub size: RcCell<i32>,
    pub idfs: RcCell<Vec<Identifier>>,
}
impl NameFactory {
    pub fn new(base_name: String, arg_type: HybridArgType) -> Self {
        //  println!("=====NameFactory==before==={base_name:?}======{}=", line!());
        // super().__init__(base_name)
        // self.arg_type = arg_type
        // self.size = 0
        // self.idfs = []
        Self {
            base_name_factory: BaseNameFactory::new(base_name),
            arg_type,
            size: RcCell::new(0),
            idfs: RcCell::new(vec![]),
        }
    }
    // """Generate a new HybridArgumentIdf which references priv_expr and has transformed type t."""
    pub fn get_new_idf(
        &self,
        t: &RcCell<TypeName>,
        priv_expr: Option<ASTFlatten>,
    ) -> HybridArgumentIdf {
        println!(
            "===get_new_idf========{:?}==={}=",
            t.borrow().get_ast_type(),
            t.borrow().size_in_uints()
        );
        let name = self.base_name_factory.get_new_name(t, true);
        let idf = HybridArgumentIdf::new(name, t.clone(), self.arg_type.clone(), priv_expr);
        *self.size.borrow_mut() += t.borrow().size_in_uints();
        self.idfs
            .borrow_mut()
            .push(Identifier::HybridArgumentIdf(idf.clone()));
        idf
    }
    // """
    // Generate a new HybridArgumentIdf with the given name.

    // This also adds the HybridArgumentIdf to the internal list of identifiers generated by this NameFactory.
    // """
    pub fn add_idf(
        &self,
        name: String,
        t: &RcCell<TypeName>,
        priv_expr: Option<&ASTFlatten>,
    ) -> HybridArgumentIdf {
        println!(
            "===add_idf========{:?}==={}=",
            t.borrow().get_ast_type(),
            t.borrow().size_in_uints()
        );
        let idf = HybridArgumentIdf::new(
            name,
            t.clone(),
            self.arg_type.clone(),
            priv_expr.clone().cloned(),
        );
        *self.base_name_factory.count.borrow_mut() += 1;
        *self.size.borrow_mut() += t.borrow().size_in_uints();
        self.idfs
            .borrow_mut()
            .push(Identifier::HybridArgumentIdf(idf.clone()));
        idf
    }
}
