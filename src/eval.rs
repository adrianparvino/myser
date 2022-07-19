use crate::{pool::{RcValue, Pool}, value::Value, builtins::Builtins};
use core::ops::Deref;
use heapless::FnvIndexMap;

pub fn eval_list<'s, Context, const N: usize, const BUILTINS: usize, const CELLS: usize>(
    context: &mut Context,
    pool: &'s Pool<'s, N>,
    cells: &mut Cells<'s, CELLS>,
    builtins: &Builtins<'s, Context, N, BUILTINS>,
    list: RcValue<'s>
) -> RcValue<'s> {
    match list.deref() {
        Value::Cons(car, cdr) => {
            let car_ = eval(context, pool, cells, builtins, car.clone());
            let cdr_ = eval_list(context, pool, cells, builtins, cdr.clone());

            pool.new_cons(car_, cdr_)
        },
        Value::Symbol("nil") => {
            pool.new_symbol("nil")
        }
        _ => panic!()
    }
}

pub struct Cells<'s, const N: usize> {
    // functions: FnvIndexMap<&'s str, &'s Value<'s>, N>,
    values: FnvIndexMap<&'s str, RcValue<'s>, N>
}

impl<'s: 'cells, 'cells, const N: usize> Cells<'s, N> {
    pub fn new() -> Self {
        Cells {
            // functions: FnvIndexMap::new(),
            values: FnvIndexMap::new(),
        }
    }

    pub fn add_value(&mut self, key: &'s str, value: RcValue<'s>) {
        self.values.insert(key, value).unwrap();
    }
}

pub fn eval<'cells, 's: 'cells, Context, const N: usize, const BUILTINS: usize, const CELLS: usize>(
    context: &mut Context,
    pool: &'s Pool<'s, N>,
    cells: &'cells mut Cells<'s, CELLS>,
    builtins: &Builtins<'s, Context, N, BUILTINS>,
    ast: RcValue<'s>
) -> RcValue<'s> {
    match ast.deref() {
        Value::Cons(car, ast) => {
            match car.deref() {
                Value::Symbol("progn") => {
                    let mut ast = ast;

                    let mut result = pool.new_symbol("nil");

                    while let Value::Cons(car, cdr) = ast.deref() {
                        result = eval(context, pool, cells, builtins, car.clone());
                        ast = &cdr;
                    }

                    result
                }
                Value::Symbol("let-") => {
                    if let Value::Cons(binding, ast) = ast.deref() {
                        if let Value::Cons(key, value) = binding.deref() {
                            if let Value::Symbol(key) = key.deref() {
                                let mut ast = ast;
                                let value = eval(context, pool, cells, builtins, value.clone());

                                let old_value = cells.values.get(key).map(|x| x.clone());
                                cells.values.insert(key, value).unwrap();

                                let mut result = pool.new_symbol("nil");

                                while let Value::Cons(car, cdr) = ast.deref() {
                                    result = eval(context, pool, cells, builtins, car.clone());
                                    ast = cdr;
                                }

                                if let Some(old_value) = old_value {
                                    cells.values.insert(key, old_value).unwrap();
                                } else {
                                    cells.values.remove(key).unwrap();
                                }

                                return result;
                            }
                        }
                    }

                    panic!()
                },
                Value::Symbol("set") => {
                    if let Value::Cons(key, ast) = ast.deref() {
                        if let Value::Cons(value, _) = ast.deref() {
                            if let Value::Symbol(key) = key.deref() {
                                let value = eval(context, pool, cells, builtins, value.clone());
                                cells.add_value(key, value);

                                return pool.new_symbol("nil")
                            }
                        }
                    }

                    panic!()
                },
                Value::Symbol("while") => {
                    if let Value::Cons(condition, ast) = ast.deref() {
                        while {
                            match *eval(context, pool, cells, builtins, condition.clone()) {
                                Value::Integer(0) => false,
                                Value::Symbol("nil") => false,
                                _ => true
                            }
                        }{
                            let mut ast = ast;

                            while let Value::Cons(car, cdr) = ast.deref() {
                                eval(context, pool, cells, builtins, car.clone());
                                ast = cdr;
                            }
                        }

                        return pool.new_symbol("nil");
                    }

                    panic!()
                },
                Value::Symbol("nil") => {
                    return pool.new_symbol("nil");
                },
                Value::Symbol(builtin) => {
                    if let Some(f) = builtins.get(builtin) {
                        let rest = eval_list(context, pool, cells, builtins, ast.clone());

                        return f(context, pool, rest);
                    }

                    return pool.new_symbol("nil");
                },
                _ => panic!()
            }
        }
        Value::Integer(n) => pool.new_integer(*n),
        Value::Symbol(symbol) => cells.values.get(symbol).map(|x| x.clone()).unwrap_or_else(|| pool.new_symbol("nil")),
        _ => panic!()
    }
}
