use crate::command;
use std::fmt;

// https://www.gnupg.org/documentation/manuals/assuan/Server-responses.html#Server-responses
#[derive(PartialEq, Debug)]
pub enum Response<'a> {
    // Request was successful.
    Ok(Option<&'a str>),

    // Request could not be fulfilled. The possible error codes are defined by libgpg-error.
    Err((&'a str, Option<&'a str>)),

    // Informational output by the server, which is still processing the request.
    // A client may not send such lines to the server while processing an Inquiry command.
    // keyword shall start with a letter or an underscore.
    S((&'a str, &'a str)),

    // Raw data returned to client. There must be exactly one space after the ’D’.
    // The values for ’%’, CR and LF must be percent escaped; these are encoded as %25, %0D and %0A, respectively.
    // Only uppercase letters should be used in the hexadecimal representation.
    // Other characters may be percent escaped for easier debugging.
    // All Data lines are considered one data stream up to the OK or ERR response.
    // Status and Inquiry Responses may be mixed with the Data lines.
    D(&'a str),

    // The server needs further information from the client.
    // The client should respond with data (using the “D” command and terminated by “END”).
    // Alternatively, the client may cancel the current operation by responding with “CAN”.
    Inquire((&'a str, &'a str)),

    // Comment line issued only for debugging purposes.
    // Totally ignored.
    Comment(Option<&'a str>),

    Custom((&'a str, Option<&'a str>)),
}

impl fmt::Display for Response<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::D(v) => write!(f, "{} {}", command::D, v),

            Response::S((k, v)) => write!(f, "{} {} {}", command::S, k, v),
            Response::Inquire((k, v)) => write!(f, "{} {} {}", command::INQUIRE, k, v),

            Self::Comment(None) => write!(f, "{}", command::COMMENT),
            Self::Comment(Some(v)) => write!(f, "{} {}", command::COMMENT, v),

            Response::Ok(None) => write!(f, "{}", command::OK),
            Response::Ok(Some(v)) => write!(f, "{} {}", command::OK, v),

            Response::Err((id, None)) => write!(f, "{} {}", command::ERR, id),
            Response::Err((id, Some(v))) => write!(f, "{} {} {}", command::ERR, id, v),

            Response::Custom((s, None)) => write!(f, "{}", s),
            Response::Custom((s, Some(v))) => write!(f, "{} {}", s, v),
        }
    }
}

impl<'a> From<&'a str> for Response<'a> {
    fn from(input: &'a str) -> Self {
        let command_and_parameters = match input.split_once(' ') {
            None => (input, None),
            Some((a, "")) => (a.trim(), None),
            Some((a, b)) => (a.trim(), Some(b.trim())),
        };

        if command_and_parameters.0[..1].eq(command::COMMENT) {
            return Self::Comment(command_and_parameters.1);
        }

        println!("{}", input);
        match command_and_parameters {
            (command::OK, v) => Self::Ok(v),
            (command::D, Some(p)) => Self::D(p),

            (command::ERR, Some(p)) => match p.split_once(' ') {
                None => Self::Err((p, None)),
                Some((e, "")) => Self::Err((e, None)),
                Some((e, v)) => Self::Err((e, Some(v))),
            },

            (command::INQUIRE, Some(p)) => match p.split_once(' ') {
                None => Self::Custom(command_and_parameters),
                Some((_, "")) => Self::Custom(command_and_parameters),
                Some((k, v)) => Self::Inquire((k, v)),
            },

            (command::S, Some(p)) => match p.split_once(' ') {
                None => Self::Custom(command_and_parameters),
                Some((_, "")) => Self::Custom(command_and_parameters),
                Some((k, v)) => Self::S((k, v)),
            },

            (command, parameters) => Self::Custom((command, parameters)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::command;
    use crate::response::Response;

    #[test]
    fn test_response_from() {
        assert_eq!(Response::from(command::OK), Response::Ok(None));
        assert_eq!(
            Response::from(format!("{} {}", command::OK, "data").as_str()),
            Response::Ok(Some("data")),
        );

        assert_eq!(
            Response::from(command::ERR),
            Response::Custom((command::ERR, None))
        );
        assert_eq!(
            Response::from(format!("{} {}", command::ERR, "id").as_str()),
            Response::Err(("id", None))
        );
        assert_eq!(
            Response::from(format!("{} {} {}", command::ERR, "id", "a description").as_str()),
            Response::Err(("id", Some("a description")))
        );

        assert_eq!(
            Response::from(command::S),
            Response::Custom((command::S, None))
        );
        assert_eq!(
            Response::from(format!("{} {}", command::S, "keyword").as_str()),
            Response::Custom((command::S, Some("keyword")))
        );
        assert_eq!(
            Response::from(
                format!("{} {} {}", command::S, "keyword", "status information").as_str()
            ),
            Response::S(("keyword", "status information"))
        );

        assert_eq!(
            Response::from(command::INQUIRE),
            Response::Custom((command::INQUIRE, None)),
        );
        assert_eq!(
            Response::from(format!("{} {}", command::INQUIRE, "keyword").as_str()),
            Response::Custom((command::INQUIRE, Some("keyword")))
        );
        assert_eq!(
            Response::from(format!("{} {} {}", command::INQUIRE, "keyword", "params").as_str()),
            Response::Inquire(("keyword", "params"))
        );

        assert_eq!(
            Response::from(command::D),
            Response::Custom((command::D, None)),
        );
        assert_eq!(
            Response::from(format!("{} {}", command::D, "some data").as_str()),
            Response::D("some data"),
        );

        assert_eq!(Response::from(command::COMMENT), Response::Comment(None),);
        assert_eq!(
            Response::from(format!("{} {}", command::COMMENT, "comment data").as_str()),
            Response::Comment(Some("comment data")),
        );
    }
}
