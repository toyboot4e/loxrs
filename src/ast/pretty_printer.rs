//! Pretty prints expression

// TODO: indent for nested blocks
// TODO: use ::std::fmt::Display

use crate::ast::expr::*;
use crate::runtime::obj::{LoxFn, LoxObj, LoxValue};

pub trait PrettyPrint {
    fn pretty_print(&self) -> String;
}

impl PrettyPrint for Expr {
    fn pretty_print(&self) -> String {
        use Expr::*;
        match *self {
            Literal(ref l) => l.pretty_print(),
            Unary(ref u) => u.pretty_print(),
            Binary(ref b) => b.pretty_print(),
            Logic(ref b) => b.pretty_print(),
            Grouping(ref expr) => expr.pretty_print(),
            Variable(ref var) => format!("{}", var.name),
            Assign(ref a) => a.pretty_print(),
            Call(ref call) => call.pretty_print(),
        }
    }
}

// Implemented for operators
trait PrettyPrintHelper {
    fn pretty_print_help(&self) -> &str;
}

impl PrettyPrintHelper for UnaryOper {
    fn pretty_print_help(&self) -> &str {
        use UnaryOper::*;
        match *self {
            Not => "!",
            Minus => "-",
        }
    }
}

impl PrettyPrintHelper for BinaryOper {
    fn pretty_print_help(&self) -> &str {
        use BinaryOper::*;
        match *self {
            Minus => "-",
            Plus => "+",
            Mul => "*",
            Div => "/",
            Equal => "=",
            NotEqual => "!=",
            Less => "<",
            LessEqual => "<=",
            Greater => ">",
            GreaterEqual => ">=",
        }
    }
}

impl PrettyPrintHelper for LogicOper {
    fn pretty_print_help(&self) -> &str {
        use LogicOper::*;
        match *self {
            Or => "or",
            And => "and",
        }
    }
}

impl PrettyPrint for LiteralData {
    fn pretty_print(&self) -> String {
        use LiteralData::*;
        match *self {
            Nil => "Nil".into(),
            Bool(b) => {
                if b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            StringLit(ref s) => format!("\"{}\"", s),
            Number(n) => n.to_string(),
        }
    }
}

impl PrettyPrint for UnaryData {
    fn pretty_print(&self) -> String {
        format!(
            "({} {})",
            self.oper.pretty_print_help(),
            self.expr.pretty_print()
        )
    }
}

impl PrettyPrint for BinaryData {
    fn pretty_print(&self) -> String {
        format!(
            "({} {} {})",
            self.oper.pretty_print_help(),
            self.left.pretty_print(),
            self.right.pretty_print()
        )
    }
}

impl PrettyPrint for LogicData {
    fn pretty_print(&self) -> String {
        format!(
            "({} {} {})",
            self.oper.pretty_print_help(),
            self.left.pretty_print(),
            self.right.pretty_print()
        )
    }
}

impl PrettyPrint for GroupData {
    fn pretty_print(&self) -> String {
        format!("group {}", self.expr.pretty_print())
    }
}

impl PrettyPrint for AssignData {
    fn pretty_print(&self) -> String {
        format!(
            "(set \"{}\" {})",
            self.assigned.name,
            self.expr.pretty_print()
        )
    }
}

impl PrettyPrint for CallData {
    fn pretty_print(&self) -> String {
        format!(
            "(call {} {})",
            self.callee.pretty_print(),
            match self.args {
                Some(ref vec) => vec
                    .iter()
                    .map(|expr| expr.pretty_print())
                    .collect::<Vec<_>>()
                    .join(", "),
                None => "()".to_string(),
            }
        )
    }
}

use crate::ast::stmt::*;

impl PrettyPrint for BlockArgs {
    fn pretty_print(&self) -> String {
        self.stmts
            .iter()
            .map(|s| s.pretty_print())
            .collect::<Vec<String>>()
            .join("\n  ")
    }
}

fn vec_to_s(xs: &Vec<impl ::std::fmt::Debug>) -> String {
    format!(
        "({})",
        xs.iter()
            .map(|x| format!("{:?}", x))
            .collect::<Vec<_>>()
            .join(", ".into())
    )
}

impl PrettyPrint for Stmt {
    fn pretty_print(&self) -> String {
        use Stmt::*;
        match *self {
            Expr(ref expr) => format!("(eval {})", expr.pretty_print()),
            Print(ref print) => format!("(print {})", print.expr.pretty_print()),
            Var(ref var) => format!("(var {} {})", var.name, var.init.pretty_print()),
            If(ref if_) => format!(
                "(if {} {} {})",
                if_.condition.pretty_print(),
                if_.if_true.pretty_print(),
                match if_.if_false {
                    Some(ref stmt) => stmt.pretty_print(),
                    None => "None".to_string(),
                }
            ),
            Block(ref block) => format!(
                "(progn {})",
                block.stmts.iter().map(|s| s.pretty_print()).fold(
                    // add newline and indent
                    "".to_string(),
                    |x, y| format!("{}\n{}", x, y)
                )
            ),
            Return(ref ret) => format!("(return {})", ret.expr.pretty_print()),
            While(ref while_) => format!(
                "(while {} {})",
                while_.condition.pretty_print(),
                while_.block.pretty_print(),
            ),
            Fn(ref f) => format!(
                "(defn {} {} ({}))",
                f.name,
                f.params
                    .as_ref()
                    .map(|params| self::vec_to_s(params))
                    .unwrap_or("()".into()),
                f.body.pretty_print(),
            ),
        }
    }
}

impl PrettyPrint for LoxValue {
    fn pretty_print(&self) -> String {
        match *self {
            LoxValue::Nil => "Nil".into(),
            LoxValue::Bool(b) => {
                if b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            LoxValue::StringLit(ref s) => format!("\"{}\"", s.clone()),
            LoxValue::Number(n) => n.to_string(),
        }
    }
}

impl PrettyPrint for LoxObj {
    fn pretty_print(&self) -> String {
        match self {
            LoxObj::Value(value) => value.pretty_print(),
            LoxObj::Callable(call) => call.pretty_print(),
        }
    }
}

impl PrettyPrint for LoxFn {
    fn pretty_print(&self) -> String {
        "(fn)".into()
    }
}

/// Tests expression printing
#[cfg(test)]
mod test {
    /// Tests this: (* (- 123) (group 45.67))
    #[test]
    fn test_in_part_5() {
        use crate::ast::expr::*;
        use crate::ast::pretty_printer::*;
        println!(
            "{}",
            Expr::binary(
                Expr::unary(UnaryOper::Minus, Expr::literal(123.0.into())),
                BinaryOper::Mul,
                Expr::group(Expr::literal(45.67.into())),
            )
            .pretty_print()
        );
    }
}
