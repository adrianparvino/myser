use crate::{pool::{Pool, RcValue}, value::Value};
use core::ops::Deref;
use heapless::FnvIndexMap;

pub fn add<'s, Context, const N: usize>(_: &mut Context, pool: &'s Pool<'s, N>, args: RcValue<'s>) -> RcValue<'s> {
    let mut args = args.deref();
    let mut result = Value::Integer(0);

    loop {
        match args {
            Value::Cons(car, cdr) => {
                result = match (car.deref(), result) {
                    (Value::Integer(car), Value::Integer(n)) => {
                        Value::Integer(car + n)
                    },
                    (Value::Number(car), Value::Integer(n)) => {
                        Value::Number(car + n as f64)
                    },
                    (Value::Integer(car), Value::Number(n)) => {
                        Value::Number(*car as f64 + n)
                    },
                    (Value::Number(car), Value::Number(n)) => {
                        Value::Number(car + n)
                    }
                    _ => {
                        panic!()
                    }
                };

                args = &cdr;
            },
            Value::Symbol("nil") => { return match result {
                Value::Integer(result) => pool.new_integer(result),
                Value::Number(result) => pool.new_number(result),
                _ => panic!()
            }},
            _ => {
                return pool.new_symbol("nil");
            }
        }
    }
}

pub fn sub<'s, Context, const N: usize>(_: &mut Context, pool: &'s Pool<'s, N>, args: RcValue<'s>) -> RcValue<'s> {
    match args.deref() {
        Value::Cons(car, args) => {
            if let Value::Symbol("nil") = args.deref() {
                match car.deref() {
                    Value::Integer(n) => {
                        return pool.new_integer(-n);
                    },
                    Value::Number(x) => {
                        return pool.new_number(-x);
                    },
                    _ => {
                        return pool.new_symbol("nil");
                    }
                }
            }

            let mut result = match car.deref() {
                Value::Integer(n) =>  Value::Integer(*n),
                Value::Number(x) =>  Value::Number(*x),
                _ => panic!()
            };
            let mut args = args.deref();

            loop {
                match args {
                    Value::Cons(car, cdr) => {
                        result = match (car.deref(), result) {
                            (Value::Integer(car), Value::Integer(n)) => {
                                Value::Integer(n - car)
                            },
                            (Value::Number(car), Value::Integer(n)) => {
                                Value::Number(n as f64 - car)
                            },
                            (Value::Integer(car), Value::Number(n)) => {
                                Value::Number(n - *car as f64)
                            },
                            (Value::Number(car), Value::Number(n)) => {
                                Value::Number(n - car)
                            }
                            _ => {
                                panic!()
                            }
                        };

                        args = &cdr;
                    },
                    Value::Symbol("nil") => { return match result {
                        Value::Integer(result) => pool.new_integer(result),
                        Value::Number(result) => pool.new_number(result),
                        _ => panic!()
                    }},
                    _ => {
                        return pool.new_symbol("nil");
                    }
                }
            }
        },
        _ => pool.new_integer(0)
    }
}

pub fn times<'s, Context, const N: usize>(_: &mut Context, pool: &'s Pool<'s, N>, args: RcValue<'s>) -> RcValue<'s> {
    let mut args = args.deref();
    let mut result = Value::Integer(1);

    loop {
        match args {
            Value::Cons(car, cdr) => {
                result = match (car.deref(), result) {
                    (Value::Integer(car), Value::Integer(n)) => {
                        Value::Integer(car * n)
                    },
                    (Value::Number(car), Value::Integer(n)) => {
                        Value::Number(car * n as f64)
                    },
                    (Value::Integer(car), Value::Number(n)) => {
                        Value::Number(*car as f64 * n)
                    },
                    (Value::Number(car), Value::Number(n)) => {
                        Value::Number(car * n)
                    }
                    _ => {
                        panic!()
                    }
                };

                args = &cdr;
            },
            Value::Symbol("nil") => { return match result {
                Value::Integer(result) => pool.new_integer(result),
                Value::Number(result) => pool.new_number(result),
                _ => panic!()
            }},
            _ => {
                return pool.new_symbol("nil");
            }
        }
    }
}

pub fn div<'s, Context, const N: usize>(_: &mut Context, pool: &'s Pool<'s, N>, args: RcValue<'s>) -> RcValue<'s> {
    match args.deref() {
        Value::Cons(car, args) => {
            if let Value::Symbol("nil") = args.deref() {
                match car.deref() {
                    Value::Integer(_) => {
                        return pool.new_integer(0);
                    },
                    Value::Number(x) => {
                        return pool.new_number(1.0/x);
                    },
                    _ => {
                        return pool.new_symbol("nil");
                    }
                }
            }

            let mut result = match car.deref() {
                Value::Integer(n) =>  Value::Integer(*n),
                Value::Number(x) =>  Value::Number(*x),
                _ => panic!()
            };
            let mut args = args.deref();

            loop {
                match args {
                    Value::Cons(car, cdr) => {
                        result = match (car.deref(), result) {
                            (Value::Integer(car), Value::Integer(n)) => {
                                Value::Integer(n / car)
                            },
                            (Value::Number(car), Value::Integer(n)) => {
                                Value::Number(n as f64 / car)
                            },
                            (Value::Integer(car), Value::Number(n)) => {
                                Value::Number(n / *car as f64)
                            },
                            (Value::Number(car), Value::Number(n)) => {
                                Value::Number(n / car)
                            }
                            _ => {
                                panic!()
                            }
                        };

                        args = &cdr;
                    },
                    Value::Symbol("nil") => { return match result {
                        Value::Integer(result) => pool.new_integer(result),
                        Value::Number(result) => pool.new_number(result),
                        _ => panic!()
                    }},
                    _ => {
                        return pool.new_symbol("nil");
                    }
                }
            }
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
        this.add("*", times);
        this.add("/", div);

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
