use crate::command::Command;
use std::fmt;

// https://www.gnupg.org/documentation/manuals/assuan/Client-requests.html#Client-requests
#[derive(PartialEq, Debug)]
pub enum Request {
    // Lines beginning with a # or empty lines are ignored.
    // This is useful to comment test scripts.
    Comment(Option<String>),

    // Sends raw data to the server. There must be exactly one space after the ’D’.
    // The values for ’%’, CR and LF must be percent escaped.
    // These are encoded as %25, %0D and %0A, respectively.
    // Only uppercase letters should be used in the hexadecimal representation.
    // Other characters may be percent escaped for easier debugging.
    // All Data lines are considered one data stream up to the OK or ERR response.
    // Status and Inquiry Responses may be mixed with the Data lines.
    D(String),

    // Close the connection.
    // The server will respond with OK.
    Bye,

    // Reset the connection but not any existing authentication.
    // The server should release all resources associated with the connection.
    Reset,

    // Used by a client to mark the end of raw data.
    // The server may send END to indicate a partial end of data.
    End,

    // Lists all commands that the server understands as comment lines on the status channel.
    Help,

    // Reserved for future extensions.
    Quit,

    // Set options for the connection. The syntax of such a line is
    //     OPTION name [ [=] value ]
    // Leading and trailing spaces around name and value are allowed but should be ignored.
    // For compatibility reasons, name may be prefixed with two dashes.
    // The use of the equal sign is optional but suggested if value is given.
    Option((String, Option<String>)),

    // This command is reserved for future extensions.
    Cancel,

    Nop,

    Unknown((String, Option<String>)),
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bye => write!(f, "{}", Command::Bye),
            Self::Reset => write!(f, "{}", Command::Reset),
            Self::End => write!(f, "{}", Command::End),
            Self::Help => write!(f, "{}", Command::Help),
            Self::Quit => write!(f, "{}", Command::Quit),
            Self::Cancel => write!(f, "{}", Command::Cancel),
            Self::Nop => write!(f, "{}", Command::Nop),

            Self::D(v) => write!(f, "{} {}", Command::D, v),

            Self::Comment(None) => write!(f, "{}", Command::Comment),
            Self::Comment(Some(v)) => write!(f, "{} {}", Command::Comment, v),

            Self::Option((k, None)) => write!(f, "{} {}", Command::Option, k),
            Self::Option((k, Some(v))) => write!(f, "{} {}={}", Command::Option, k, v),

            Self::Unknown((c, None)) => write!(f, "{}", c),
            Self::Unknown((c, Some(p))) => write!(f, "{} {}", c, p),
        }
    }
}

impl From<&str> for Request {
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

        let command = Command::try_from(command_and_parameters.0.as_ref());
        if command.is_err() {
            return Self::Unknown(command_and_parameters);
        }

        match (command.unwrap(), command_and_parameters.clone().1) {
            (Command::Bye, _) => Self::Bye,
            (Command::Reset, _) => Self::Reset,
            (Command::End, _) => Self::End,
            (Command::Help, _) => Self::Help,
            (Command::Quit, _) => Self::Quit,

            (Command::Option, Some(arg)) => match arg.split_once('=') {
                Some((k, v)) => Self::Option((k.trim().into(), Some(v.trim().into()))),
                None => match arg.split_once(' ') {
                    Some((k, v)) => Self::Option((k.trim().into(), Some(v.trim().into()))),
                    None => Self::Option((arg.trim().into(), None)),
                },
            },

            (Command::Cancel, _) => Self::Cancel,
            (Command::Nop, _) => Self::Nop,

            (Command::D, Some(p)) => Self::D(p),
            (_, _) => Self::Unknown(command_and_parameters),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::command::Command;
    use crate::request::Request;

    #[test]
    fn test_request_from() {
        assert_eq!(Request::from(Command::Bye.as_ref()), Request::Bye);
        assert_eq!(Request::from(Command::Reset.as_ref()), Request::Reset);
        assert_eq!(Request::from(Command::End.as_ref()), Request::End);
        assert_eq!(Request::from(Command::Help.as_ref()), Request::Help);
        assert_eq!(Request::from(Command::Quit.as_ref()), Request::Quit);
        assert_eq!(Request::from(Command::Cancel.as_ref()), Request::Cancel);
        assert_eq!(Request::from(Command::Nop.as_ref()), Request::Nop);

        assert_eq!(Request::from("#"), Request::Comment(None));
        assert_eq!(
            Request::from("# some content"),
            Request::Comment(Some("some content".into()))
        );
        assert_eq!(
            Request::from("#### some content"),
            Request::Comment(Some("### some content".into()))
        );

        assert_eq!(
            Request::from("OPTION"),
            Request::Unknown(("OPTION".into(), None))
        );
        assert_eq!(
            Request::from("OPTION option"),
            Request::Option(("option".into(), None))
        );
        assert_eq!(
            Request::from("OPTION option value"),
            Request::Option(("option".into(), Some("value".into())))
        );
        assert_eq!(
            Request::from("OPTION option=value"),
            Request::Option(("option".into(), Some("value".into())))
        );
        assert_eq!(
            Request::from("OPTION option    =  value"),
            Request::Option(("option".into(), Some("value".into())))
        );

        assert_eq!(Request::from("D"), Request::Unknown(("D".into(), None)));
        assert_eq!(Request::from("D with data"), Request::D("with data".into()));

        assert_eq!(
            Request::from("UNKNOWN"),
            Request::Unknown(("UNKNOWN".into(), None))
        );
    }
}
