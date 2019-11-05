# loxrs

Yet another hobby project to follow the book [Crafting Interpreters](http://www.craftinginterpreters.com/) in Rust.

## Progress

I'm doing part II (treewalk interpreter). Done: Ch.12 [Classes](https://craftinginterpreters.com/classes.html).

### TODO

- Challenges 
- Better error context
- Add `+=` etc. 
- PrettyPrint with indent 
- Ch. 13 (inheritance) 

## Example

### Runnning a File

Do `cargo run -- <filename>` to run a program.

> Some keywords have different names from the original.

```rust
class Vec2 {
    fn init(x, y) {
        @.x = x;
        @.y = y;
    }

    fn add(another) {
        return Vec2(@.x + another.x, @.y + another.y);
    }
}

var x = Vec2(1, 2);
var y = Vec2(3, 4);
print x.add(y);
```

### Debug output of AST

You get debug output when `-d` or `--debug` is specified. It contains a pretty-printed AST:

```sh
$ cargo run -- examples/syntax/class_self.lox --debug | sed -n '/^===== AST/,/^$/p'
===== AST =====
0 (class TestClass
    (defn test_fn ()
        (print "x y z")
        (print @)
        (eval (set @ x 13))
        (print (get x @))))
1 (print (TestClass ()))
2 (eval ((get test_fn (TestClass ())) ()))
3 (print @)

```

### REPL

```sh
$ cargo run
Entered loxrs REPL (q or Ctrl-c for quit)
> var x = 3;
Ok(Value(Nil)
> print x;
Ok(Value(Nil))
>
```

## Layout of the source code

```sh
$ cd src; tree
.
├── analizer
│   ├── mod.rs
│   └── resolver.rs
├── ast
│   ├── expr.rs
│   ├── mod.rs
│   ├── pretty_printer.rs
│   ├── stmt.rs
│   └── visitor.rs
├── lexer
│   ├── mod.rs
│   ├── parser.rs
│   ├── scanner.rs
│   └── token.rs
├── lib.rs
├── main.rs
└── runtime
    ├── env.rs
    ├── interpreter.rs
    ├── mod.rs
    └── obj.rs

4 directories, 17 files

```

## Notes on the implementation

### Differences from the original Lox

- Change: variable declaration requires initial value expression 
- Change: `while` without parentheses 
- Change: `return` is dealt as `Ok(Some(LoxObj))`, not as an exception 
- Skipped: `for` statement (maybe make range-based one instead later) 
- Skipped: forbidding to return something from a constructor 

### Rust specigic tips (for me)

- structuring 
    - visualizing dependencies and decoupling the runtime (treewalk) from the lexer 
- lexer (both scanner and parser, in this repository) 
    - binary-based source iterator like ron? 
    - using [itertools](https://docs.rs/itertools/0.8.0/itertools/)::multipeek for Scanner 
    - using [Box](https://doc.rust-lang.org/std/boxed/struct.Box.html) to make `struct`s [Sized](https://doc.rust-lang.org/std/marker/trait.Sized.html) 
        - where to place `Box`?: in a super node (I chose) or sub nodes? 
    - right recursive parsing with higher order functions 
        - efficiency? 
- runtime (treewalk) 
    - using visitor pattern vs just `match` to AST 
    - using concrete types rather than wrapping them with a `Stmt` 

