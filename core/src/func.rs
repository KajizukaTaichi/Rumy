use crate::*;

#[derive(Clone, Debug)]
pub enum Func {
    BuiltIn(fn(Value, &mut Engine) -> Result<Value, Fault>),
    UserDefined(String, Box<Expr>, Type),
}

impl Func {
    pub fn parse(source: &str) -> Result<Self, Fault> {
        // Lambda abstract that original formula in the theory
        if source.starts_with("λ") && source.contains(".") {
            let source = remove!(source, "λ");
            Self::common(&source)
        // Lambda abstract using back-slash instead of lambda mark
        } else if source.starts_with("\\") && source.contains(".") {
            let source = remove!(source, "\\");
            Self::common(&source)
        } else {
            Err(Fault::Syntax)
        }
    }

    fn common(source: &str) -> Result<Self, Fault> {
        let (arg, body) = ok!(source.split_once("."))?;
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Fault::Syntax);
        }
        let splited_body = tokenize(body, ["->"].as_slice(), false);
        let (arg, body, annotation) =
            if let (Some((arg, ano_arg)), Ok(body)) = (arg.split_once(":"), splited_body) {
                let ano_ret = ok!(body.last())?;
                let body = join!(ok!(body.get(..body.len() - 1))?, "->");
                (
                    arg,
                    body,
                    Some(Box::new((Type::parse(ano_arg)?, Type::parse(ano_ret)?))),
                )
            } else {
                (arg, body.to_string(), None)
            };
        if !is_identifier(arg) {
            return Err(Fault::Syntax);
        }
        Ok(Func::UserDefined(
            arg.to_string(),
            Box::new(Expr::parse(&body)?),
            Type::Func(annotation),
        ))
    }

    pub fn bind(&self, anno: Type) -> Result<Self, Fault> {
        let Func::UserDefined(arg, body, _) = self else {
            return Err(Fault::Syntax);
        };
        if let Type::Func(Some(inner)) = anno {
            Ok(Func::UserDefined(
                arg.to_owned(),
                if let Expr::Value(Value::Func(func)) = *body.clone() {
                    Box::new(Expr::Value(Value::Func(func.bind(inner.1.clone())?)))
                } else {
                    body.clone()
                },
                Type::Func(Some(Box::new((inner.0, inner.1)))),
            ))
        } else {
            Ok(self.clone())
        }
    }
}
