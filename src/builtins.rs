use crate::{constants::{ZERO, NIL_SYM}, pool::Pool, value::Value};
use heapless::FnvIndexMap;

pub fn add<'s, Context, const N: usize>(_: &mut Context, pool: &'s Pool<'s, N>, mut args: &'s Value<'s>) -> &'s Value<'s> {
    let mut result = 0;

    loop {
        match args {
            Value::Cons(Value::Integer(car), cdr) => {
                result += car;
                args = cdr;
            },
            Value::Symbol("nil") => { return pool.new_integer(result); },
            _ => {
                return &NIL_SYM;
            }
        }
    }
}

pub fn sub<'s, Context, const N: usize>(_: &mut Context, pool: &'s Pool<'s, N>, args: &'s Value<'s>) -> &'s Value<'s> {
    match args {
        Value::Cons(Value::Integer(car), Value::Symbol("nil")) => {
            return pool.new_integer(-car);
        },
        Value::Cons(&Value::Integer(car), mut args) => {
            let mut result = car;

            loop {
                match args {
                    Value::Cons(Value::Integer(car), cdr) => {
                        result -= car;
                        args = cdr;
                    },
                    Value::Symbol("nil") => { return pool.new_integer(result); },
                    _ => {
                        return &NIL_SYM;
                    }
                }
            }
        },
        _ => &ZERO
    }
}

pub type Builtin<'s, Context, const N: usize> = fn(context: &mut Context, pool: &'s Pool<'s, N>, list: &'s Value<'s>) -> &'s Value<'s>;

pub struct Builtins<'s, Context, const N: usize, const M: usize> {
    map: FnvIndexMap<&'s str, Builtin<'s, Context, N>, M>
}

impl <'s, Context, const N: usize, const M: usize> Builtins<'s, Context, N, M> {
    pub fn new() -> Self {
        let map = FnvIndexMap::new();
        let mut this = Self { map };
        this.add("+", add as Builtin<'s, Context, N>);
        this.add("-", sub as Builtin<'s, Context, N>);

        return this;
    }

    pub fn add(&mut self, key: &'s str, builtin: Builtin<'s, Context, N>) {
        if let Err(_) = self.map.insert(key, builtin) {
            panic!()
        }
    }

    pub fn get(&self, key: &'_ str) -> Option<&'_ Builtin<'s, Context, N>> {
        self.map.get(key)
    }
}
