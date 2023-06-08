use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PageParsingError {
    PageNotFoundError,
    NotSuccessfulPageError,
}

impl fmt::Display for PageParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PageParsingError::PageNotFoundError => write!(f, "Got a 404, page not found."),
            PageParsingError::NotSuccessfulPageError => write!(
                f,
                "Didn't get confirmation page for successful registration."
            ),
        }
    }
}

impl Error for PageParsingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            PageParsingError::PageNotFoundError => None,
            PageParsingError::NotSuccessfulPageError => None,
        }
    }
}
