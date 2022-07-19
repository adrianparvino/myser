#![feature(as_array_of_cells)]

use myser::{
    builtins::{Builtin, Builtins},
    eval::{eval, Cells},
    parser::parse,
    pool::{RcValue, Pool},
    value::Value,
};
use core::ops::Deref;
use std::io::Write;

trait HasStdin {
    fn stdin(&self) -> &std::io::Stdin;
}

trait HasStdout {
    fn stdout(&self) -> &std::io::Stdout;
}

struct Context {
    stdin: std::io::Stdin,
    stdout: std::io::Stdout,
}

impl HasStdin for Context {
    fn stdin(&self) -> &std::io::Stdin {
        &self.stdin
    }
}

impl HasStdout for Context {
    fn stdout(&self) -> &std::io::Stdout {
        &self.stdout
    }
}

impl Context {
    fn new(stdin: std::io::Stdin, stdout: std::io::Stdout) -> Self {
        Self { stdin, stdout }
    }
}

fn print<'s, Context: HasStdout, const N: usize>(context: &mut Context, pool: &'s Pool<'s, N>, args: RcValue<'s>) -> RcValue<'s> {
    if let Value::Cons(car, cdr) = args.deref() {
        if let Value::Symbol("nil") = cdr.deref() {
            write!(context.stdout(), "{:?}\n", car).unwrap();

            return pool.new_symbol("nil");
        }
    }

    write!(context.stdout(), "{:?}\n", args).unwrap();
    pool.new_symbol("nil")
}

fn read<'s, Context: HasStdin, const N: usize>(context: &mut Context, pool: &'s Pool<'s, N>, _: RcValue<'s>) -> RcValue<'s> {
    let mut buffer = String::new();
    context.stdin().read_line(&mut buffer).unwrap();

    if let Ok(n) = buffer.trim().parse() {
        return pool.new_integer(n);
    }

    pool.new_symbol("nil")
}

fn main() {
    let mut buffer = String::new();

    let pool: Pool<'_, 10000> = Pool::new();
    let mut builtins: Builtins<'_, _, 10000, 16> = Builtins::new();
    builtins.add("print", print as Builtin<'_, _, 10000>);
    builtins.add("read", read as Builtin<'_, _, 10000>);

    let mut context = Context::new(std::io::stdin(), std::io::stdout());
    let mut cells: Cells<'_, 16> = Cells::new();

    std::io::stdin().read_line(&mut buffer).unwrap();

    let result = parse(&pool, &buffer).unwrap().1;
    println!("{:?}", result);
    // println!("{:?}", pool.used());
    eval(&mut context, &pool, &mut cells, &builtins, result);
    // println!("{:?}", result);
    // println!("{:?}", eval(&pool, result, &builtins));

    // println!("{}", pool.used());
    // println!("{:?}", pool.new_cons(&ZERO, &ONE));
    // println!("{:?}", eval(&pool.new_cons(&PLUS, pool.new_cons(&TWO, pool.new_cons(&PLUS, pool.new_cons(&TWO, &ONE))))));
}
