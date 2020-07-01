# loxrs tree-walk interpreter

Done: Ch.12 [Classes](https://craftinginterpreters.com/classes.html)

## Note

I refactored a bit, but still this is a terrible implementation.

It would be easier to just slice an input string and that's actually done in the Part 3 of the book.

## Example

### Runnning a File

You can run a program such as this:

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

> Some keywords have different names from the original.

Let's run it:

```rust
$ cargo run -- examples/for_readme.lox
(instance (class Vec2) ((x 4), (y 6)))
```

### Debug output of AST

When `-d` or `--debug` is specified, you get debug output, which contains a pretty-printed AST:

```sh
$ cargo run -- examples/for_readme.lox --debug | sed -n '/^===== AST/,/^$/p'
===== AST =====
0 (class Vec2
    (defn init (x, y)
        (eval (set @ x x))
        (eval (set @ y y)))
    (defn add (another)
        (return (Vec2 ((+ (get x @) (get x another)), (+ (get y @) (get y another)))))))
1 (var x (Vec2 (1, 2)))
2 (var y (Vec2 (3, 4)))
3 (print ((get add x) (y)))

```

### REPL

Of cource we also have a read–eval–print loop:

```sh
$ cargo run
Entered loxrs REPL (press q<Enter> or Ctrl-c to quit)
> var x = 3;
> print x;
3
>
```

## Notes

### Dependent crates

* [itertools](https://github.com/rust-itertools/itertools) to multipeek
* [anyhow](https://crates.io/crates/anyhow) and [thiserror](https://github.com/dtolnay/thiserror) for CLI error handling

### Differences from the original Lox

- implementation
    - `return` is dealt as `Ok(Some(LoxObj))`, not as an exception
- design
    - variable declaration requires initial value expression
    - `while` without parentheses

#### Skipped

- there's no `for` statement (maybe make range-based one instead later)
- can't `return` from a constructor

## TODO

I won't do but these are left:

- Challenges
- Ch. 13 (inheritance)
- Better error context (source position tracking and error chain)
- Add `+=` `-=` etc.
