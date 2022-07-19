#![feature(as_array_of_cells)]

use myser::{
    builtins::{Builtin, Builtins},
    eval::{eval, Cells},
    parser::parse,
    pool::Pool,
    value::Value,
    constants::NIL_SYM
};

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

fn print<'s, Context: HasStdout, const N: usize>(context: &mut Context, _: &'s Pool<'s, N>, args: &'s Value<'s>) -> &'s Value<'s> {
    match args {
        Value::Cons(car, Value::Symbol("nil")) => { write!(context.stdout(), "{:?}\n", car).unwrap(); },
        args => { write!(context.stdout(), "{:?}\n", args).unwrap(); }
    }

    &NIL_SYM
}

fn read<'s, Context: HasStdin, const N: usize>(context: &mut Context, pool: &'s Pool<'s, N>, _: &'s Value<'s>) -> &'s Value<'s> {
    let mut buffer = String::new();
    context.stdin().read_line(&mut buffer).unwrap();

    if let Ok(n) = buffer.trim().parse() {
        return pool.new_integer(n);
    }

    &NIL_SYM
}

fn main() {
    let mut buffer = String::new();

    let pool: Pool<'_, 100> = Pool::new();
    let mut builtins: Builtins<'_, _, 100, 16> = Builtins::new();
    builtins.add("print", print as Builtin<'_, _, 100>);
    builtins.add("read", read as Builtin<'_, _, 100>);

    let mut context = Context::new(std::io::stdin(), std::io::stdout());
    let mut cells: Cells<'_, 16> = Cells::new();

    std::io::stdin().read_line(&mut buffer).unwrap();

    let result = parse(&pool, &buffer).unwrap().1;
    // println!("{:?}", result);
    // println!("{:?}", pool.used());
    eval(&mut context, &pool, &mut cells, &builtins, result);
    // println!("{:?}", result);
    // println!("{:?}", eval(&pool, result, &builtins));

    // println!("{}", pool.used());
    // println!("{:?}", pool.new_cons(&ZERO, &ONE));
    // println!("{:?}", eval(&pool.new_cons(&PLUS, pool.new_cons(&TWO, pool.new_cons(&PLUS, pool.new_cons(&TWO, &ONE))))));
}
