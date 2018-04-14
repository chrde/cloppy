use std::fmt::{self, Display};
use failure::{
    Backtrace,
    Context,
    Fail,
};

#[derive(Debug)]
struct MyError {
    inner: Context<MyErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum MyErrorKind {
    #[fail(display = "Error while calling winapi, {}", _0)]
    WindowsError(&'static str),
    #[fail(display = "Error while processing user settings.")]
    UserSettingsError,
    #[fail(display = "Error while processing change journal events.")]
    UsnJournalError,
    #[fail(display = "UsnRecord v{} is not supported", _0)]
    UsnRecordVersionUnsupported(u16),
}

//Boilerplate start
impl Fail for MyError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl MyError {
    pub fn kind(&self) -> MyErrorKind {
        *self.inner.get_context()
    }
}

impl From<MyErrorKind> for MyError {
    fn from(kind: MyErrorKind) -> MyError {
        MyError { inner: Context::new(kind) }
    }
}

impl From<Context<MyErrorKind>> for MyError {
    fn from(inner: Context<MyErrorKind>) -> MyError {
        MyError { inner }
    }
}
//Boilerplate end
