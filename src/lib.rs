pub mod loader;

pub trait ExpectErr<T> {
    fn expect_else_err(self) -> T;
}
impl<T, E: std::fmt::Display> ExpectErr<T> for Result<T, E> {
    fn expect_else_err(self) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => panic!("{}", err),
        }
    }
}
