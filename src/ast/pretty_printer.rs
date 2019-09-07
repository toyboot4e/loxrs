//! Pretty prints expression
//!
// TODO: use ::std::fmt::Display
use super::expr::*;

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
            Variable(ref name) => format!("(var {})", name),
            Assign(ref a) => a.pretty_print(),
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
            Or => "||",
            And => "&&",
        }
    }
}

impl PrettyPrint for LiteralArgs {
    fn pretty_print(&self) -> String {
        use LiteralArgs::*;
        match *self {
            Nil => "Nil".into(),
            Bool(b) => {
                if b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            StringL(ref s) => s.clone(),
            Number(n) => n.to_string(),
        }
    }
}

impl PrettyPrint for UnaryArgs {
    fn pretty_print(&self) -> String {
        format!(
            "({} {})",
            self.oper.pretty_print_help(),
            self.expr.pretty_print()
        )
    }
}

impl PrettyPrint for BinaryArgs {
    fn pretty_print(&self) -> String {
        format!(
            "({} {} {})",
            self.oper.pretty_print_help(),
            self.left.pretty_print(),
            self.right.pretty_print()
        )
    }
}

impl PrettyPrint for LogicArgs {
    fn pretty_print(&self) -> String {
        format!(
            "({} {} {})",
            self.oper.pretty_print_help(),
            self.left.pretty_print(),
            self.right.pretty_print()
        )
    }
}

impl PrettyPrint for GroupingArgs {
    fn pretty_print(&self) -> String {
        format!("group {}", self.expr.pretty_print())
    }
}

impl PrettyPrint for AssignArgs {
    fn pretty_print(&self) -> String {
        format!("(assign \"{}\" {})", self.name, self.expr.pretty_print())
    }
}

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

use crate::ast::stmt::*;
impl PrettyPrint for Stmt {
    fn pretty_print(&self) -> String {
        use Stmt::*;
        match *self {
            Expr(ref expr) => format!("(expr {})", expr.pretty_print()),
            Print(ref print) => format!("(print {})", print.expr.pretty_print()),
            Var(ref var) => format!("(var {} {})", var.name, var.init.pretty_print()),
            If(ref if_) => format!(
                "(if ({}) {} {})",
                if_.condition.pretty_print(),
                if_.if_true.pretty_print(),
                match if_.if_false {
                    Some(ref stmt) => stmt.pretty_print(),
                    None => "None".to_string(),
                }
            ),
            Block(ref stmts) => format!(
                "(block {})",
                stmts
                    .iter()
                    .map(|s| s.pretty_print())
                    .collect::<Vec<String>>()
                    .join("\n  ")
            ),
        }
    }
}
