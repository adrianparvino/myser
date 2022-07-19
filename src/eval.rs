use crate::{constants::NIL_SYM, pool::Pool, value::Value, builtins::Builtins};
use heapless::FnvIndexMap;

pub fn eval_list<'s, Context, const N: usize, const BUILTINS: usize, const CELLS: usize>(
    context: &mut Context,
    pool: &'s Pool<'s, N>,
    cells: &mut Cells<'s, CELLS>,
    builtins: &Builtins<'s, Context, N, BUILTINS>,
    list: &'s Value<'s>
) -> &'s Value<'s> {
    match list {
        cons @ &Value::Cons(car, cdr) => {
            let car_ = eval(context, pool, cells, builtins, car);
            let cdr_ = eval_list(context, pool, cells, builtins, cdr);

            if car as *const Value<'s> == car_ as *const Value<'s>
            && cdr as *const Value<'s> == cdr_ as *const Value<'s> {
                    return cons;
            }

            pool.new_cons(car_, cdr_)
        },
        nil @ Value::Symbol("nil") => {
            nil
        }
        _ => panic!()
    }
}

pub struct Cells<'s, const N: usize> {
    // functions: FnvIndexMap<&'s str, &'s Value<'s>, N>,
    values: FnvIndexMap<&'s str, &'s Value<'s>, N>
}

impl<'s: 'cells, 'cells, const N: usize> Cells<'s, N> {
    pub fn new() -> Self {
        Cells {
            // functions: FnvIndexMap::new(),
            values: FnvIndexMap::new(),
        }
    }

    pub fn add_value(&mut self, key: &'s str, value: &'s Value<'s>) {
        self.values.insert(key, value).unwrap();
    }
}

pub fn eval<'cells, 's: 'cells, Context, const N: usize, const BUILTINS: usize, const CELLS: usize>(
    context: &mut Context,
    pool: &'s Pool<'s, N>,
    cells: &'cells mut Cells<'s, CELLS>,
    builtins: &Builtins<'s, Context, N, BUILTINS>,
    ast: &'s Value<'s>
) -> &'s Value<'s> {
    match ast {
        Value::Cons(Value::Symbol("progn"), mut ast) => {
            let mut result = &NIL_SYM;

            while let Value::Cons(car, cdr) = ast {
                result = eval(context, pool, cells, builtins, car);
                ast = cdr;
            }

            result
        },
        Value::Cons(Value::Symbol("let-"), Value::Cons(Value::Cons(Value::Symbol(key), value), mut ast)) => {
            let value = eval(context, pool, cells, builtins, value);
            let mut result = &NIL_SYM;

            let old_value = cells.values.get(key).map(|x| *x);
            cells.values.insert(key, value).unwrap();

            while let Value::Cons(car, cdr) = ast {
                result = eval(context, pool, cells, builtins, car);
                ast = cdr;
            }

            if let Some(old_value) = old_value {
                cells.values.insert(key, old_value).unwrap();
            } else {
                cells.values.remove(key).unwrap();
            }

            result
        },
        Value::Cons(Value::Symbol("def"), Value::Cons(Value::Symbol(key), Value::Cons(ast, Value::Symbol("nil")))) => {
            let value = eval(context, pool, cells, builtins, ast);
            cells.add_value(key, value);

            &NIL_SYM
        },
        Value::Cons(Value::Symbol("while"), Value::Cons(condition, ast)) => {
            while {
                match eval(context, pool, cells, builtins, condition) {
                    Value::Integer(0) => false,
                    Value::Symbol("nil") => false,
                    _ => true
                }
            }{
                let mut ast = ast;

                while let Value::Cons(car, cdr) = ast {
                    eval(context, pool, cells, builtins, car);
                    ast = cdr;
                }
            }

            &NIL_SYM
        }
        Value::Cons(Value::Symbol(builtin), rest) => {
            if let Some(f) = builtins.get(builtin) {
                let rest = eval_list(context, pool, cells, builtins, rest);

                return f(context, pool, rest);
            }

            return &NIL_SYM;
        }
        n @ Value::Integer(_) => n,
        nil @ Value::Symbol("nil") => nil,
        Value::Symbol(symbol) => cells.values.get(symbol).map(|x| *x).unwrap_or(&NIL_SYM),
        Value::Cons(car, cdr) => pool.new_cons(
            eval(context, pool, cells, builtins, car),
            eval(context, pool, cells, builtins, cdr),
        ),
        _ => panic!()
    }
}
