#[macro_use] extern crate error_type;

error_type! {
    #[derive(Debug)]
    pub enum AppError {
        Io(std::io::Error) { cause; },
        Simple(std::borrow::Cow<'static, str>) {
            desc (e) &**e;
            from (s: &'static str) s.into();
            from (s: String) s.into();
        },
        Other(Box<std::error::Error>) {
            desc (e) e.description();
            cause (e) Some(&**e);
        }
    }
}

#[test]
fn test() {
    use std::error::Error;

    type E = AppError;
    macro_rules! err {
        ($e:expr) => (From::from($e));
    }

    let _: E = err!(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"));
    let _: E = err!("Test string");
    let _: E = err!(format!("Another test string"));
    let _: E = err!(std::borrow::Cow::Borrowed("Hi!"));
    
    let e: E = err!("Where's the Kaboom?!");
    let e: Box<Error> = From::from(e);
    assert_eq!(&format!("{}", e.description()), "Where's the Kaboom?!");

    let e: E = err!(e);
    assert_eq!(&format!("{}", e.description()), "Where's the Kaboom?!");
    
    let e: E = err!("Hello, World!");
    assert_eq!(format!("{}", e), "Hello, World!");
    assert_eq!(format!("{:?}", e), r#"Simple("Hello, World!")"#);
    assert_eq!(format!("{}", e.description()), "Hello, World!");
}
