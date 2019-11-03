# loxrs

Yet another hobby project to follow the book [Crafting Interpreters](http://www.craftinginterpreters.com/) in Rust.

I'm doing part II (treewalk interpreter). Done: Ch.12 [Classes](https://craftinginterpreters.com/classes.html)

## Example

Do `cargo run -- <filename>` to run a program.

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

## Notes on the implementation

**Those text below are just for me at the moment**.

### Differences from the original Lox

- Change: variable declaration requires initial value expression 
- Change: `while` without parentheses 
- Change: `return` is dealt as `Ok(Some(LoxObj))`, not as an exception 
- Skipped: `for` statement (maybe make range-based one instead later) 
- Skipped: forbidding to return something from a constructor 

### Rust specigic tips (for me)

- structuring 
    - decoupling the runtime (treewalk) from the lexer 
- lexer (both scanner and parser, in this repository) 
    - binary-based source iterator? 
    - using [itertools](https://docs.rs/itertools/0.8.0/itertools/)::multipeek for Scanner 
    - using [Box](https://doc.rust-lang.org/std/boxed/struct.Box.html) to make `struct`s [Sized](https://doc.rust-lang.org/std/marker/trait.Sized.html) 
        - where to place `Box`?: in a super node (I chose) or sub nodes? 
    - right recursive parsing with higher order functions 
        - efficiency? 
- runtime (treewalk) 
    - using visitor pattern vs just `match` to AST 

## TODO

- part II 
- challenges 
- add `+=` etc. 
- PrettyPrint with indent 
- cache just for `VarUseId`, then resolve `@` 

