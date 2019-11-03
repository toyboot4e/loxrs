use crate::ast::expr::Expr;
use ::std::rc::Rc;

// TODO: use proper places for function definitions
pub type Params = Vec<String>;
pub type FnBody = Rc<Vec<Stmt>>;

/// Stmt → expr | if | print | block ;
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    /// Just evaluate the expression
    Expr(Expr),
    Fn(FnDeclArgs),
    Print(PrintArgs),
    Var(VarDeclArgs),
    If(Box<IfArgs>),
    Return(Return),
    While(WhileArgs),
    /// A non-negeric separated code block, not a body of a function
    Block(BlockArgs),
    Class(ClassDeclArgs),
}

impl Stmt {
    pub fn expr(expr: Expr) -> Self {
        Stmt::Expr(expr)
    }

    pub fn print(expr: Expr) -> Self {
        Stmt::Print(PrintArgs { expr: expr })
    }

    pub fn var_dec(name: String, init: Expr) -> Self {
        Stmt::Var(VarDeclArgs::new(name, init))
    }

    pub fn if_then_else(condition: Expr, then: BlockArgs, else_: Option<ElseBranch>) -> Self {
        Stmt::If(Box::new(IfArgs {
            condition: condition,
            if_true: then,
            if_false: else_,
        }))
    }

    pub fn block(stmts: Vec<Stmt>) -> Self {
        Stmt::Block(BlockArgs { stmts: stmts })
    }

    /// Even if a function returns nothing, it returns `Some(LoxObj::Nul)` internally
    pub fn return_(expr: Expr) -> Self {
        Stmt::Return(Return { expr: expr })
    }

    pub fn while_(condition: Expr, block: BlockArgs) -> Self {
        Stmt::While(WhileArgs {
            condition: condition,
            block: block,
        })
    }
}

impl From<PrintArgs> for Stmt {
    fn from(item: PrintArgs) -> Self {
        Stmt::Print(item)
    }
}

impl From<VarDeclArgs> for Stmt {
    fn from(item: VarDeclArgs) -> Self {
        Stmt::Var(item)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintArgs {
    // pub message: String,
    pub expr: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarDeclArgs {
    pub name: String,
    pub init: Expr,
}

// split into IfThen and IfThenElse
#[derive(Clone, Debug, PartialEq)]
pub struct IfArgs {
    pub condition: Expr,
    // branches
    /// True branch
    pub if_true: BlockArgs,
    /// Else branch. If it's `if`, the branch means `if else`.
    pub if_false: Option<ElseBranch>,
}

impl IfArgs {
    pub fn new(cond: Expr, if_true: BlockArgs, if_false: Option<ElseBranch>) -> Self {
        Self {
            condition: cond,
            if_true: if_true,
            if_false: if_false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElseBranch {
    JustElse(BlockArgs),
    ElseIf(Box<IfArgs>),
}

impl ElseBranch {
    pub fn else_if(if_: impl Into<IfArgs>) -> Self {
        ElseBranch::ElseIf(Box::new(if_.into()))
    }
}

impl VarDeclArgs {
    /// Unlike the original Lox language, loxrs always requires initializer for declarations
    pub fn new(name: String, init: Expr) -> Self {
        Self {
            name: name,
            init: init,
        }
    }
}

/// A separated code block, not a body of a function
///
/// Newtype pattern for `Vec<Stmt>>`.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockArgs {
    pub stmts: Vec<Stmt>,
}

impl BlockArgs {
    pub fn into_stmt(self) -> Stmt {
        Stmt::Block(self)
    }
}

/// Even if a function returns nothing, it returns `Some(LoxObj::Nul)` internally
#[derive(Clone, Debug, PartialEq)]
pub struct Return {
    pub expr: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhileArgs {
    pub condition: Expr,
    pub block: BlockArgs,
}

/// Function definition translated to AST
#[derive(Clone, Debug, PartialEq)]
pub struct FnDeclArgs {
    pub name: String,
    pub body: FnBody,
    pub params: Params,
}

impl FnDeclArgs {
    pub fn new(name: String, body: Rc<Vec<Stmt>>, params: impl Into<Params>) -> Self {
        Self {
            name: name,
            body: body,
            params: params.into(),
        }
    }
}

/// In Lox, fields are dynamically added
#[derive(Clone, Debug, PartialEq)]
pub struct ClassDeclArgs {
    pub name: String,
    pub methods: Vec<FnDeclArgs>,
}

impl ClassDeclArgs {
    pub fn new(name: String, methods: Vec<FnDeclArgs>) -> Self {
        Self {
            name: name,
            methods: methods,
        }
    }
}
