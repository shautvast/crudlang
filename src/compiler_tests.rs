#[cfg(test)]
mod tests {
    use crate::compile;
    use crate::scanner::scan;

    #[test]
    fn literal_int() {
        assert!(compile("1").is_ok());
    }

    #[test]
    fn literal_float() {
        assert!(compile("2.1").is_ok());
    }

    #[test]
    fn literal_float_scientific() {
        assert!(compile("2.1e5").is_ok());
    }

    #[test]
    fn literal_string() {
        assert!(compile(r#""a""#).is_ok());
    }

    #[test]
    fn literal_list() {
        assert!(compile(r#"["abc","def"]"#).is_ok());
    }

    #[test]
    fn let_infer_type() {
        assert!(compile(r#"let a=1"#).is_ok());
    }

    #[test]
    fn let_u32() {
        assert!(compile(r#"let a:u32=1"#).is_ok());
    }

    #[test]
    fn let_char() {
        assert!(scan(r#"let a:char='a'"#).is_ok());
    }

    #[test]
    fn let_u32_invalid_value_negative() {
        let r = compile("let a:u32=-1");
        assert!(r.is_err());
        if let Err(e) = &r {
            assert_eq!(
                e.to_string(),
                "error at line 1: Incompatible types. Expected u32, found i32/64"
            );
        }
    }

    #[test]
    fn let_u64_invalid_value_negative() {
        let r = compile("let a:u64=-1");
        assert!(r.is_err());
        if let Err(e) = &r {
            assert_eq!(
                e.to_string(),
                "error at line 1: Incompatible types. Expected u64, found i32/64"
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
                "error at line 1: Incompatible types. Expected u64, found string"
            );
        }
    }

    #[test]
    fn call_fn_with_args_returns_value() {
        assert!(
            compile(
                r#"
fn hello(name: string) -> string:
    "Hello " + name
hello("world")"#
            )
            .is_ok()
        );
    }
}
