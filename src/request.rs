use crate::command;
use std::fmt;

// https://www.gnupg.org/documentation/manuals/assuan/Client-requests.html#Client-requests
#[derive(PartialEq, Debug)]
pub enum Request<'a> {
    // Lines beginning with a # or empty lines are ignored.
    // This is useful to comment test scripts.
    Comment(Option<&'a str>),

    // Sends raw data to the server. There must be exactly one space after the ’D’.
    // The values for ’%’, CR and LF must be percent escaped.
    // These are encoded as %25, %0D and %0A, respectively.
    // Only uppercase letters should be used in the hexadecimal representation.
    // Other characters may be percent escaped for easier debugging.
    // All Data lines are considered one data stream up to the OK or ERR response.
    // Status and Inquiry Responses may be mixed with the Data lines.
    D(&'a str),

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
    Option((&'a str, Option<&'a str>)),

    // This command is reserved for future extensions.
    Cancel,

    Nop,

    Unknown((&'a str, Option<&'a str>)),
}

impl fmt::Display for Request<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bye => write!(f, "{}", command::BYE),
            Self::Reset => write!(f, "{}", command::RESET),
            Self::End => write!(f, "{}", command::END),
            Self::Help => write!(f, "{}", command::HELP),
            Self::Quit => write!(f, "{}", command::QUIT),
            Self::Cancel => write!(f, "{}", command::CANCEL),
            Self::Nop => write!(f, "{}", command::NOP),

            Self::D(v) => write!(f, "{} {}", command::D, v),

            Self::Comment(None) => write!(f, "{}", command::COMMENT),
            Self::Comment(Some(v)) => write!(f, "{} {}", command::COMMENT, v),

            Self::Option((k, None)) => write!(f, "{} {}", command::OPTION, k),
            Self::Option((k, Some(v))) => write!(f, "{} {}={}", command::OPTION, k, v),

            Self::Unknown((c, None)) => write!(f, "{}", c),
            Self::Unknown((c, Some(p))) => write!(f, "{} {}", c, p),
        }
    }
}

impl<'a> From<&'a str> for Request<'a> {
    fn from(input: &'a str) -> Self {
        let command_and_parameters = match input.split_once(' ') {
            None => (input, None),
            Some((a, "")) => (a.trim(), None),
            Some((a, b)) => (a.trim(), Some(b.trim())),
        };

        if command_and_parameters.0[..1].eq(command::COMMENT) {
            return Self::Comment(command_and_parameters.1);
        }

        match command_and_parameters {
            (command::BYE, _) => Self::Bye,
            (command::RESET, _) => Self::Reset,
            (command::END, _) => Self::End,
            (command::HELP, _) => Self::Help,
            (command::QUIT, _) => Self::Quit,

            (command::OPTION, Some(arg)) => match arg.split_once('=') {
                Some((k, v)) => Self::Option((k, Some(v))),
                None => match arg.split_once(' ') {
                    Some((k, v)) => Self::Option((k, Some(v))),
                    None => Self::Option((arg, None)),
                },
            },

            (command::CANCEL, _) => Self::Cancel,
            (command::NOP, _) => Self::Nop,

            (command::D, Some(p)) => Self::D(p),
            (command, parameters) => Self::Unknown((command, parameters)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::command;
    use crate::request::Request;

    #[test]
    fn test_request_from() {
        assert_eq!(Request::from(command::BYE), Request::Bye);
        assert_eq!(Request::from(command::RESET), Request::Reset);
        assert_eq!(Request::from(command::END), Request::End);
        assert_eq!(Request::from(command::HELP), Request::Help);
        assert_eq!(Request::from(command::QUIT), Request::Quit);
        assert_eq!(Request::from(command::CANCEL), Request::Cancel);
        assert_eq!(Request::from(command::NOP), Request::Nop);

        assert_eq!(Request::from(command::COMMENT), Request::Comment(None));
        assert_eq!(
            Request::from(format!("{} {}", command::COMMENT, "some content").as_str()),
            Request::Comment(Some("some content"))
        );

        assert_eq!(
            Request::from(command::OPTION),
            Request::Unknown((command::OPTION, None))
        );
        assert_eq!(
            Request::from(format!("{} {}", command::OPTION, "OPTION").as_str()),
            Request::Option(("OPTION", None))
        );
        assert_eq!(
            Request::from(format!("{} {} {}", command::OPTION, "OPTION", "VALUE").as_str()),
            Request::Option(("OPTION", Some("VALUE")))
        );
        assert_eq!(
            Request::from(format!("{} {}={}", command::OPTION, "OPTION", "VALUE").as_str()),
            Request::Option(("OPTION", Some("VALUE")))
        );

        assert_eq!(
            Request::from(command::D),
            Request::Unknown((command::D, None))
        );
        assert_eq!(
            Request::from(format!("{} {}", command::D, "DATA").as_str()),
            Request::D("DATA")
        );

        assert_eq!(
            Request::from("UNKNOWN"),
            Request::Unknown(("UNKNOWN", None))
        );
    }
}
