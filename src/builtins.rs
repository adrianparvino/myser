use crate::{pool::{Pool, RcValue}, value::Value};
use core::ops::Deref;
use heapless::FnvIndexMap;

pub fn add<'s, Context, const N: usize>(_: &mut Context, pool: &'s Pool<'s, N>, mut args: RcValue<'s>) -> RcValue<'s> {
    let mut result = 0;

    loop {
        match args.deref() {
            Value::Cons(car, cdr) => {
                if let Value::Integer(car) = car.deref() {
                    result += car;
                    args = cdr.clone();
                }
            },
            Value::Symbol("nil") => { return pool.new_integer(result); },
            _ => {
                return pool.new_symbol("nil");
            }
        }
    }
}

pub fn sub<'s, Context, const N: usize>(_: &mut Context, pool: &'s Pool<'s, N>, args: RcValue<'s>) -> RcValue<'s> {
    match args.deref() {
        Value::Cons(car, args) => {
            if let Value::Integer(car) = car.deref() {
                if let  Value::Symbol("nil") = args.deref() {
                    return pool.new_integer(-car);
                } else {
                    let mut args = args;
                    let mut result = *car;

                    loop {
                        match args.deref() {
                            Value::Cons(car, cdr) => {
                                if let Value::Integer(car) = car.deref() {
                                    result -= car;
                                    args = cdr;
                                } else {
                                    return pool.new_integer(result);
                                }
                            },
                            Value::Symbol("nil") => { return pool.new_integer(result); },
                            _ => {
                                return pool.new_symbol("nil");
                            }
                        }
                    }
                }
            }
            pool.new_integer(0)
        },
        _ => pool.new_integer(0)
    }
}

pub type Builtin<'s, Context, const N: usize> = fn(context: &mut Context, pool: &'s Pool<'s, N>, list: RcValue<'s>) -> RcValue<'s>;

pub struct Builtins<'s, Context, const N: usize, const BUILTINS: usize> {
    map: FnvIndexMap<&'s str, Builtin<'s, Context, N>, BUILTINS>,
}

impl <'s, Context, const N: usize, const BUILTINS: usize> Builtins<'s, Context, N, BUILTINS> {
    pub fn new() -> Self {
        let map = FnvIndexMap::new();
        let mut this = Self { map };
        this.add("+", add);
        this.add("-", sub);

        return this;
    }

    pub fn add(&mut self, key: &'s str, builtin: Builtin<'s, Context, N>) {
        if let Err(_) = self.map.insert(key, builtin) {
            panic!()
        }
    }

    pub fn get(&self, key: &'_ str) -> Option<&Builtin<'s, Context, N>> {
        self.map.get(key)
    }
}
