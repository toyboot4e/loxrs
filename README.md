# loxrs

Yet another hobby project to follow the book [Crafting Interpreters](http://www.craftinginterpreters.com/) in Rust.

Now on: Ch.11 [Resolving and Binding](https://craftinginterpreters.com/resolving-and-binding.html)

## Examples

We can run lox program such as:

```rust
// prints 1 2 3 .. n
fn count(n) {
  if n <= 0 { return; }
  if n > 1 { count(n - 1); }
  print n;
}

count(3);
```

## Overview of the book

To be written.

- What will be implemented and what will be not? 
- Part 2: tree-walk interpreter 
    - Recursive descent parer 

## Notes on the implementation

### Differences from the original Lox

- Change: variable declaration requires initial value expression 
- Change: `while` without parentheses 
- Change: using early returs instead of making `return` as an exception 
- Skipped: `for` statement (maybe make range-based one later instead) 

### Rust specigic tips (for me)

- structuring 
    - decoupling the runtime (treewalk) from the lexer 
- lexer (scanner and parser) 
    - binary-based source iterator? 
    - using [itertools](https://docs.rs/itertools/0.8.0/itertools/)::multipeek for Scanner 
    - using [Box](https://doc.rust-lang.org/std/boxed/struct.Box.html) to make `struct`s [Sized](https://doc.rust-lang.org/std/marker/trait.Sized.html) 
        - where to place `Box`?: in a super node (I chose) or sub nodes? 
    - right recursive parsing with higher order functions 
        - efficiency? 
- runtime (treewalk) 
    - using visitor pattern vs just `match` to statements 

### Performance

To be written.

- Can we reduce the number of cloning without using references? 
    - And is it really better for performance? 
- When cloning is done in loxrs? 
    - AST -> runtime object: 
        - when defining functions: clones all the statemenst in the block 

## TODO
- part 2 
- challenges 
