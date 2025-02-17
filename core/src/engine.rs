use crate::*;

#[derive(Debug, Clone)]
pub struct Engine {
    pub scope: IndexMap<String, Value>,
    pub effect: IndexSet<String>,
    pub is_toplevel: bool,
    pub is_lazy: bool,
    pub mode: Mode,
}

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    Pure,
    Effect,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            mode: Mode::Pure,
            is_toplevel: true,
            is_lazy: false,
            effect: IndexSet::from(["eval".to_string()]),
            scope: IndexMap::from([
                (
                    "stdlib".to_string(),
                    Value::Str("https://kajizukataichi.github.io/MLtalk/lib/std.ml".to_string()),
                ),
                (
                    "eval".to_string(),
                    Value::Func(Func::BuiltIn(|expr, engine| {
                        Ok(Block::parse(&expr.get_str()?)?.eval(engine)?)
                    })),
                ),
                (
                    "type".to_string(),
                    Value::Func(Func::BuiltIn(|expr, _| Ok(Value::Type(expr.type_of())))),
                ),
                (
                    "alphaConvert".to_string(),
                    Value::Func(Func::BuiltIn(|args, _| {
                        let args = args.get_list()?;
                        let func = ok!(args.first(), Fault::ArgLen)?;
                        let new_name = ok!(args.get(1), Fault::ArgLen)?.get_str()?;
                        let Value::Func(Func::UserDefined(old_name, body, anno)) = func else {
                            return Err(Fault::Type(func.to_owned(), Type::Func(None, Mode::Pure)));
                        };
                        Ok(Value::Func(Func::UserDefined(
                            new_name.clone(),
                            Box::new(body.replace(
                                &Expr::Refer(old_name.to_owned()),
                                &Expr::Refer(new_name),
                            )),
                            anno.clone(),
                        )))
                    })),
                ),
            ]),
        }
    }

    pub fn alloc(&mut self, name: &String, value: &Value) -> Result<(), Fault> {
        if is_identifier(name) {
            if name != "_" {
                self.scope.insert(name.clone(), value.clone());
            }
            Ok(())
        } else {
            Err(Fault::Syntax)
        }
    }

    pub fn access(&mut self, name: &str) -> Result<Value, Fault> {
        ok!(
            if let Mode::Pure = self.mode {
                if self.is_effective(name) {
                    return Err(Fault::Pure(name.to_string()));
                } else {
                    self.scope.get(name)
                }
            } else {
                self.scope.get(name)
            },
            Fault::Refer(name.to_owned())
        )
        .cloned()
    }

    pub fn set_effect(&mut self, name: &str) {
        self.effect.insert(name.to_string());
    }

    pub fn unset_effect(&mut self, name: &str) {
        self.effect.shift_remove(name);
    }

    pub fn is_effective(&self, name: &str) -> bool {
        self.effect.contains(&name.to_string())
    }
}
