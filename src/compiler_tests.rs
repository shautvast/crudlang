#[cfg(test)]
mod tests {
    use crate::value::Value;
    use crate::{compile, run};

    #[test]
    fn literal_int() {
        assert_eq!(run("1"), Ok(Value::I64(1)));
    }

    #[test]
    fn literal_float() {
        assert_eq!(run("2.1"), Ok(Value::F64(2.1)));
    }

    #[test]
    fn literal_float_scientific() {
        assert_eq!(run("2.1e5"), Ok(Value::F64(2.1e5)));
    }

    #[test]
    fn literal_string() {
        assert_eq!(run(r#""a""#), Ok(Value::String("a".into())));
    }

    #[test]
    fn literal_list() {
        assert_eq!(run(r#"["abc","def"]"#), Ok(Value::List(vec![Value::String("abc".into()), Value::String("def".into())])));
    }

    #[test]
    fn infer_type() {
        assert_eq!(run(r#"let a=1
a"#), Ok(Value::I64(1)));
    }

    #[test]
    fn define_u32() {
        assert_eq!(run(r#"let a:u32=1
a"#), Ok(Value::U32(1)));
    }

    #[test]
    fn define_char() {
        assert_eq!(
            run(r#"let a:char='a'
a"#),
            Ok(Value::Char('a'))
        );
    }

    #[test]
    fn define_u32_invalid_value_negative() {
        let r = compile("let a:u32=-1");
        assert!(r.is_err());
        if let Err(e) = &r {
            assert_eq!(
                e.to_string(),
                "Compilation failed: error at line 1, Type mismatch: Expected u32, found i32/64"
            );
        }
    }

    #[test]
    fn define_u64_invalid_value_negative() {
        let r = compile("let a:u64=-1");
        assert!(r.is_err());
        if let Err(e) = &r {
            assert_eq!(
                e.to_string(),
                "Compilation failed: error at line 1, Type mismatch: Expected u64, found i32/64"
            );
        }
    }

    #[test]
    fn let_u64_invalid_value_string() {
        let r = compile(r#"let a:u64="not ok""#);
        assert!(r.is_err());
        if let Err(e) = &r {
            assert_eq!(
                e.to_string(),
                "Compilation failed: error at line 1, Type mismatch: Expected u64, found string"
            );
        }
    }

    #[test]
    fn call_fn_with_args_returns_value() {
        assert_eq!(
            run(r#"
fn add_hello(name: string) -> string:
    "Hello " + name
add_hello("world")"#,),
            Ok(Value::String("Hello world".to_string()))
        );
    }

    #[test]
    fn define_object() {
        let r = compile(
            r#"
object Person:
   name: string"#,
        );
        assert!(r.is_ok()); // does nothing runtime
    }

    //     #[test]
    //     fn object_() {
    //         let r = compile(r#"
    // object Person:
    //    name: string
    //
    // let p = Person{name: "Sander"}
    // print p
    // "#, );
    //         println!("{:?}", r);
    //         assert!(r.is_ok());
    //     }

    #[test]
    fn literal_map() {
        let result = run(r#"{"name": "Dent", "age": 40 }"#);
        assert!(result.is_ok());
        let result = result.unwrap();
        if let Value::Map(map) = result {
            assert_eq!(
                map.get(&Value::String("name".to_string())).unwrap(),
                &Value::String("Dent".to_string())
            );
            assert_eq!(
                map.get(&Value::String("age".to_string())).unwrap(),
                &Value::I64(40)
            );
        }
    }

    #[test]
    fn define_map() {
        let result = run(r#"let m = {"name": "Dent"}
m"#);

        let result = result.unwrap();
        if let Value::Map(map) = result {
            assert_eq!(
                map.get(&Value::String("name".to_string())).unwrap(),
                &Value::String("Dent".to_string())
            );
        }
    }

    #[test]
    fn keyword_error(){
        let result = run(r#"let map = {"name": "Dent"}"#);
        assert!(result.is_err());
        assert_eq!("Compilation failed: error at line 1, 'map' is a keyword. You cannot use it as an identifier",result.unwrap_err().to_string());
    }
}
