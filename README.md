# Karm Language
A simple language based on expressions and with a pure functional twist.
In my journey of understanding how compilers works, there is KARM. It's only a project but it's kind of like my baby. Just a little language, with a grammar and a syntax that I made myself.

Talking about syntax, here is the BNF of the language:
```html
<expr> ::= <fn> ';'

<fn> ::= 'fn' <id> ['::' <id>] '->' <content_expr>

<content_expr> ::= (<if> | <binary> | <literal>)

<if> ::= 'if' <content_expr> '?' <content_expr> [':' <content_expr>]

<binary> ::= (<binary> | <term>) <op> ( <binary> | <term>)

<term> ::= (<literal> | <fn_call>)

<literal> ::= ('+w/' | '+d/')

<op> ::= ('+' | '-' | '/' | '*')

<fn_call> ::= <id> ['(' ...<content_expr> ')']
``` 
### Kind of a notepad

With this grammar we can deduce all these behaviour :

#### HelloWorld
```rust
fn main -> puts("Hello, World");
```

### Example
With this example :
```rust
fn day -> 22 + 5;
```
We are supposed to get returned something like this:

```rust
Expr::Fn {
	id: "day"
	params: None,
	expr: Expr::Binary {
		op: Op::Plus,
		left: Expr::Literal("22"),
		right: Expr::Literal("5")
	}
}
```
## Ideas
Just a bunch of ideas I got that I will implement later
```rust
fn fib :: n -> if n <= 1 ? n : fib(n - 1) + fib(n - 2);
```