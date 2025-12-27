pub mod camera;
pub mod loader;
pub mod recorder;

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

pub trait PrintErr {
    fn print_err(self);
}

impl<E: std::fmt::Display> PrintErr for Result<(), E> {
    fn print_err(self) {
        match self {
            Ok(()) => {}
            Err(e) => println!("{e}"),
        }
    }
}
