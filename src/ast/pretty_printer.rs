//! Prints expression in a pretty format

use ::std::fmt::Write;

// TODO: indent for nested blocks
// TODO: use ::std::fmt::Display

impl PrettyPrint for Stmt {
    fn pretty_print(&self) -> String {
        let mut s = String::new();
        self::pretty_write_stmt(&mut s, 0, self);
        return s;
    }
}

pub fn write_indent(s: &mut String, indent: isize) {
    for _ in 0..indent {
        write!(s, "    ").unwrap();
    }
}

pub fn write_slice(s: &mut String, xs: &[impl ::std::fmt::Display]) {
    write!(s, "(").unwrap();
    if let Some((last, xs)) = xs.split_last() {
        for x in xs.iter() {
            write!(s, "{}", x).unwrap();
            write!(s, ", ").unwrap();
        }
        write!(s, "{}", last).unwrap();
    }
    write!(s, ")").unwrap();
}

pub fn write_fn(s: &mut String, indent: isize, f: &FnDeclArgs) {
    write!(s, "(defn {} ", f.name).unwrap();
    self::write_slice(s, &f.params);
    write!(s, "\n").unwrap();
    self::write_indent(s, indent + 1);
    self::write_stmts(s, indent + 1, &f.body);
    write!(s, ")").unwrap();
}

pub fn write_block(s: &mut String, indent: isize, stmts: &[Stmt]) {
    write!(s, "(block \n").unwrap();
    self::write_indent(s, indent);
    self::write_stmts(s, indent, stmts);
    write!(s, ")").unwrap();
}

pub fn write_stmts(s: &mut String, indent: isize, stmts: &[Stmt]) {
    if stmts.len() == 1 {
        self::pretty_write_stmt(s, indent, &stmts[0]);
        return;
    }
    match stmts.split_last() {
        Some((last, stmts)) => {
            for stmt in stmts {
                self::pretty_write_stmt(s, indent, stmt);
                write!(s, "\n").unwrap();
                self::write_indent(s, indent);
            }
            self::pretty_write_stmt(s, indent, last);
        }
        None => {}
    }
}

pub fn pretty_write_if(s: &mut String, indent: isize, if_: &IfArgs) {
    write!(s, "(if {} ", if_.condition.pretty_print()).unwrap();
    self::write_stmts(s, indent + 1, &if_.if_true.stmts);
    write!(s, " ").unwrap();
    match if_.if_false {
        Some(ref else_) => match else_ {
            ElseBranch::ElseIf(ref else_if) => self::pretty_write_if(s, indent + 1, &else_if),
            ElseBranch::JustElse(ref block) => {
                self::write_stmts(s, indent + 1, &block.stmts);
            }
        },
        None => write!(s, "None)").unwrap(),
    }
}

pub fn pretty_write_stmt(s: &mut String, indent: isize, stmt: &Stmt) {
    use Stmt::*;
    match *stmt {
        Expr(ref expr) => write!(s, "(eval {})", expr.pretty_print()).unwrap(),
        Print(ref print) => write!(s, "(print {})", print.expr.pretty_print()).unwrap(),
        Var(ref var) => write!(s, "(var {} {})", var.name, var.init.pretty_print()).unwrap(),
        If(ref if_) => self::pretty_write_if(s, indent + 1, if_),
        Block(ref block) => {
            write!(s, "(block ").unwrap();
            write_stmts(s, indent + 1, &block.stmts);
            write!(s, ")").unwrap();
        }
        Return(ref ret) => write!(s, "(return {})", ret.expr.pretty_print()).unwrap(),
        While(ref while_) => {
            write!(s, "(while {}\n", while_.condition.pretty_print(),).unwrap();
            self::write_indent(s, indent + 1);
            self::write_stmts(s, indent + 1, &while_.block.stmts);
            write!(s, ")").unwrap();
        }
        Fn(ref f) => write_fn(s, indent, f),
        Class(ref class) => {
            write!(s, "(class {}\n", class.name,).unwrap();
            self::write_indent(s, indent + 1);
            if let Some((last, methods)) = class.methods.split_last() {
                for method in methods.iter() {
                    self::write_fn(s, indent + 1, method);
                    write!(s, "\n").unwrap();
                    write_indent(s, indent);
                }
                self::write_fn(s, indent + 1, last);
            }
            write!(s, ")").unwrap();
        }
    }
}

pub fn pretty_vec(xs: impl IntoIterator<Item = impl ::std::fmt::Display>) -> String {
    format!(
        "({})",
        xs.into_iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join(", ".into())
    )
}

use crate::ast::{expr::*, stmt::*};

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
            Get(ref get) => get.pretty_print(),
            Set(ref set) => set.pretty_print(),
            Self_(ref self_) => self_.pretty_print(),
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
            "(assign \"{}\" {})",
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
            self::pretty_vec(self.args.iter().map(|expr| expr.pretty_print()))
        )
    }
}

impl PrettyPrint for GetUseData {
    fn pretty_print(&self) -> String {
        format!("(get {} {})", self.name, self.body.pretty_print())
    }
}

impl PrettyPrint for SetUseData {
    fn pretty_print(&self) -> String {
        format!(
            "(set {} {} {})",
            self.body.pretty_print(),
            self.name,
            self.value.pretty_print(),
        )
    }
}

// statements

impl PrettyPrint for SelfData {
    fn pretty_print(&self) -> String {
        "@".to_string()
    }
}

impl PrettyPrint for BlockArgs {
    fn pretty_print(&self) -> String {
        self.stmts
            .iter()
            .map(|s| s.pretty_print())
            .collect::<Vec<String>>()
            .join("\n  ")
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
