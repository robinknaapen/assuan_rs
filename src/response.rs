use crate::command::Command;
use crate::errors;
use std::fmt;

#[derive(PartialEq, Debug)]
pub enum ResponseErr {
    Gpg(errors::GpgErrorCode),
    Custom(errors::Custom),
}

impl fmt::Display for ResponseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gpg(s) => write!(f, "{}", s),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Response {
    // Request was successful.
    Ok(Option<String>),

    // Request could not be fulfilled. The possible error codes are defined by libgpg-error.
    Err((ResponseErr, Option<String>)),

    // Informational output by the server, which is still processing the request.
    // A client may not send such lines to the server while processing an Inquiry command.
    // keyword shall start with a letter or an underscore.
    S((String, String)),

    // Raw data returned to client. There must be exactly one space after the ’D’.
    // The values for ’%’, CR and LF must be percent escaped; these are encoded as %25, %0D and %0A, respectively.
    // Only uppercase letters should be used in the hexadecimal representation.
    // Other characters may be percent escaped for easier debugging.
    // All Data lines are considered one data stream up to the OK or ERR response.
    // Status and Inquiry Responses may be mixed with the Data lines.
    D(String),

    // The server needs further information from the client.
    // The client should respond with data (using the “D” command and terminated by “END”).
    // Alternatively, the client may cancel the current operation by responding with “CAN”.
    Inquire((String, String)),

    // Comment line issued only for debugging purposes.
    // Totally ignored.
    Comment(Option<String>),

    Custom((String, Option<String>)),
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::D(v) => write!(f, "{} {}", Command::D, v),

            Response::S((k, v)) => write!(f, "{} {} {}", Command::S, k, v),
            Response::Inquire((k, v)) => write!(f, "{} {} {}", Command::Inquire, k, v),

            Self::Comment(None) => write!(f, "{}", Command::Comment),
            Self::Comment(Some(v)) => write!(f, "{} {}", Command::Comment, v),

            Response::Ok(None) => write!(f, "{}", Command::Ok),
            Response::Ok(Some(v)) => write!(f, "{} {}", Command::Ok, v),

            Response::Err((id, None)) => write!(f, "{} {}", Command::Err, id),
            Response::Err((id, Some(v))) => write!(f, "{} {} {}", Command::Err, id, v),

            Response::Custom((s, None)) => write!(f, "{}", s),
            Response::Custom((s, Some(v))) => write!(f, "{} {}", s, v),
        }
    }
}

impl From<&str> for Response {
    fn from(input: &str) -> Self {
        let command_and_parameters = match input.split_once(' ') {
            None => (String::from(input), None),
            Some((a, "")) => (String::from(a.trim()), None),
            Some((a, b)) => (String::from(a.trim()), Some(String::from(b.trim()))),
        };

        if command_and_parameters.0[..1].eq(Command::Comment.as_ref()) {
            return match input[1..].trim() {
                "" => Self::Comment(None),
                s => Self::Comment(Some(String::from(s))),
            };
        }

        let command = Command::try_from(command_and_parameters.0.as_str());
        if command.is_err() {
            return Self::Custom(command_and_parameters);
        }

        match (command.unwrap(), command_and_parameters.clone().1) {
            (Command::Ok, v) => Self::Ok(v),
            (Command::D, Some(p)) => Self::D(p),

            (Command::Err, Some(p)) => {
                let (e, p) = match p.split_once(' ') {
                    None => (p, None),
                    Some((e, "")) => (String::from(e), None),
                    Some((e, v)) => (String::from(e), Some(String::from(v))),
                };

                let error_code = errors::GpgErrorCode::try_from(e.as_str());
                if let Ok(ec) = error_code {
                    return Self::Err((ResponseErr::Gpg(ec), p));
                }

                let error_code = errors::Custom::try_from(e.as_str());
                if let Ok(ec) = error_code {
                    return Self::Err((ResponseErr::Custom(ec), p));
                }

                Self::Err((ResponseErr::Gpg(errors::GpgErrorCode::UnknownErrno), p))
            }

            (Command::Inquire, Some(p)) => match p.split_once(' ') {
                None => Self::Custom((Command::Inquire.to_string(), Some(p))),
                Some((_, "")) => Self::Custom((Command::Inquire.to_string(), Some(p))),
                Some((k, v)) => Self::Inquire((String::from(k), String::from(v))),
            },

            (Command::S, Some(p)) => match p.split_once(' ') {
                None => Self::Custom((Command::S.to_string(), Some(p))),
                Some((_, "")) => Self::Custom((Command::S.to_string(), Some(p))),
                Some((k, v)) => Self::S((String::from(k), String::from(v))),
            },

            _ => Self::Custom(command_and_parameters),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::command::Command;
    use crate::errors;
    use crate::response::{Response, ResponseErr};

    #[test]
    fn test_response_from() {
        assert_eq!(Response::from("OK"), Response::Ok(None));
        assert_eq!(
            Response::from(format!("{} {}", Command::Ok, "data").as_str()),
            Response::Ok(Some("data".into())),
        );

        assert_eq!(
            Response::from("ERR"),
            Response::Custom(("ERR".into(), None))
        );
        assert_eq!(
            Response::from("ERR 16383"),
            Response::Err((ResponseErr::Gpg(errors::GpgErrorCode::Eof), None))
        );
        assert_eq!(
            Response::from("ERR 16383 with description"),
            Response::Err((
                ResponseErr::Gpg(errors::GpgErrorCode::Eof),
                Some("with description".into())
            ))
        );
        assert_eq!(
            Response::from(format!("ERR {} with description", (1 << 15 | 140) + 1).as_str()),
            Response::Err((
                ResponseErr::Custom(errors::Custom((1 << 15 | 140) + 1)),
                Some("with description".into())
            ))
        );

        assert_eq!(Response::from("S"), Response::Custom(("S".into(), None)));
        assert_eq!(
            Response::from("S keyword"),
            Response::Custom(("S".into(), Some("keyword".into())))
        );
        assert_eq!(
            Response::from("S keyword status information"),
            Response::S(("keyword".into(), "status information".into()))
        );

        assert_eq!(
            Response::from("INQUIRE"),
            Response::Custom(("INQUIRE".into(), None)),
        );
        assert_eq!(
            Response::from("INQUIRE keyword"),
            Response::Custom(("INQUIRE".into(), Some("keyword".into())))
        );
        assert_eq!(
            Response::from("INQUIRE keyword params"),
            Response::Inquire(("keyword".into(), "params".into()))
        );

        assert_eq!(Response::from("D"), Response::Custom(("D".into(), None)),);
        assert_eq!(
            Response::from("D some data"),
            Response::D("some data".into()),
        );

        assert_eq!(Response::from("#"), Response::Comment(None),);
        assert_eq!(
            Response::from("# comment data"),
            Response::Comment(Some("comment data".into())),
        );

        assert_eq!(Response::from("#"), Response::Comment(None),);
        assert_eq!(
            Response::from("### comment data"),
            Response::Comment(Some("## comment data".into())),
        );
    }
}
