use std::fmt;

pub const COMMAND_BYE: &str = "BYE";
pub const COMMAND_RESET: &str = "RESET";
pub const COMMAND_END: &str = "END";
pub const COMMAND_HELP: &str = "HELP";
pub const COMMAND_QUIT: &str = "QUIT";
pub const COMMAND_OPTION: &str = "OPTION";
pub const COMMAND_CANCEL: &str = "CANCEL";
pub const COMMAND_NOP: &str = "NOP";

// Response
pub const COMMAND_OK: &str = "OK";
pub const COMMAND_ERR: &str = "ERR";
pub const COMMAND_S: &str = "S";
pub const COMMAND_INQUIRE: &str = "INQUIRE";

// Request/Response
pub const COMMAND_D: &str = "D";
pub const COMMAND_COMMENT: &str = "#";

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
            Self::Bye => write!(f, "{}", COMMAND_BYE),
            Self::Reset => write!(f, "{}", COMMAND_RESET),
            Self::End => write!(f, "{}", COMMAND_END),
            Self::Help => write!(f, "{}", COMMAND_HELP),
            Self::Quit => write!(f, "{}", COMMAND_QUIT),
            Self::Cancel => write!(f, "{}", COMMAND_CANCEL),
            Self::Nop => write!(f, "{}", COMMAND_NOP),

            Self::D(v) => write!(f, "{} {}", COMMAND_D, v),

            Self::Comment(None) => write!(f, "{}", COMMAND_COMMENT),
            Self::Comment(Some(v)) => write!(f, "{} {}", COMMAND_COMMENT, v),

            Self::Option((k, None)) => write!(f, "{} {}", COMMAND_OPTION, k),
            Self::Option((k, Some(v))) => write!(f, "{} {}={}", COMMAND_OPTION, k, v),

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

        if command_and_parameters.0[..1].eq(COMMAND_COMMENT) {
            return Self::Comment(command_and_parameters.1);
        }

        match command_and_parameters {
            (COMMAND_BYE, _) => Self::Bye,
            (COMMAND_RESET, _) => Self::Reset,
            (COMMAND_END, _) => Self::End,
            (COMMAND_HELP, _) => Self::Help,
            (COMMAND_QUIT, _) => Self::Quit,

            (COMMAND_OPTION, Some(arg)) => match arg.split_once('=') {
                Some((k, v)) => Self::Option((k, Some(v))),
                None => match arg.split_once(' ') {
                    Some((k, v)) => Self::Option((k, Some(v))),
                    None => Self::Option((arg, None)),
                },
            },

            (COMMAND_CANCEL, _) => Self::Cancel,
            (COMMAND_NOP, _) => Self::Nop,

            (COMMAND_D, Some(p)) => Self::D(p),
            (command, parameters) => Self::Unknown((command, parameters)),
        }
    }
}

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
            Response::D(v) => write!(f, "{} {}", COMMAND_D, v),

            Response::S((k, v)) => write!(f, "{} {} {}", COMMAND_S, k, v),
            Response::Inquire((k, v)) => write!(f, "{} {} {}", COMMAND_INQUIRE, k, v),

            Self::Comment(None) => write!(f, "{}", COMMAND_COMMENT),
            Self::Comment(Some(v)) => write!(f, "{} {}", COMMAND_COMMENT, v),

            Response::Ok(None) => write!(f, "{}", COMMAND_OK),
            Response::Ok(Some(v)) => write!(f, "{} {}", COMMAND_OK, v),

            Response::Err((id, None)) => write!(f, "{} {}", COMMAND_ERR, id),
            Response::Err((id, Some(v))) => write!(f, "{} {} {}", COMMAND_ERR, id, v),

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

        if command_and_parameters.0[..1].eq(COMMAND_COMMENT) {
            return Self::Comment(command_and_parameters.1);
        }

        println!("{}", input);
        match command_and_parameters {
            (COMMAND_OK, v) => Self::Ok(v),
            (COMMAND_D, Some(p)) => Self::D(p),

            (COMMAND_ERR, Some(p)) => match p.split_once(' ') {
                None => Self::Err((p, None)),
                Some((e, "")) => Self::Err((e, None)),
                Some((e, v)) => Self::Err((e, Some(v))),
            },

            (COMMAND_INQUIRE, Some(p)) => match p.split_once(' ') {
                None => Self::Custom(command_and_parameters),
                Some((_, "")) => Self::Custom(command_and_parameters),
                Some((k, v)) => Self::Inquire((k, v)),
            },

            (COMMAND_S, Some(p)) => match p.split_once(' ') {
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
    use crate::*;

    #[test]
    fn test_request_from() {
        assert_eq!(Request::from(COMMAND_BYE), Request::Bye);
        assert_eq!(Request::from(COMMAND_RESET), Request::Reset);
        assert_eq!(Request::from(COMMAND_END), Request::End);
        assert_eq!(Request::from(COMMAND_HELP), Request::Help);
        assert_eq!(Request::from(COMMAND_QUIT), Request::Quit);
        assert_eq!(Request::from(COMMAND_CANCEL), Request::Cancel);
        assert_eq!(Request::from(COMMAND_NOP), Request::Nop);

        assert_eq!(Request::from(COMMAND_COMMENT), Request::Comment(None));
        assert_eq!(
            Request::from(format!("{} {}", COMMAND_COMMENT, "some content").as_str()),
            Request::Comment(Some("some content"))
        );

        assert_eq!(
            Request::from(COMMAND_OPTION),
            Request::Unknown((COMMAND_OPTION, None))
        );
        assert_eq!(
            Request::from(format!("{} {}", COMMAND_OPTION, "OPTION").as_str()),
            Request::Option(("OPTION", None))
        );
        assert_eq!(
            Request::from(format!("{} {} {}", COMMAND_OPTION, "OPTION", "VALUE").as_str()),
            Request::Option(("OPTION", Some("VALUE")))
        );
        assert_eq!(
            Request::from(format!("{} {}={}", COMMAND_OPTION, "OPTION", "VALUE").as_str()),
            Request::Option(("OPTION", Some("VALUE")))
        );

        assert_eq!(
            Request::from(COMMAND_D),
            Request::Unknown((COMMAND_D, None))
        );
        assert_eq!(
            Request::from(format!("{} {}", COMMAND_D, "DATA").as_str()),
            Request::D("DATA")
        );

        assert_eq!(
            Request::from("UNKNOWN"),
            Request::Unknown(("UNKNOWN", None))
        );
    }

    #[test]
    fn test_response_from() {
        assert_eq!(Response::from(COMMAND_OK), Response::Ok(None));
        assert_eq!(
            Response::from(format!("{} {}", COMMAND_OK, "data").as_str()),
            Response::Ok(Some("data")),
        );

        assert_eq!(
            Response::from(COMMAND_ERR),
            Response::Custom((COMMAND_ERR, None))
        );
        assert_eq!(
            Response::from(format!("{} {}", COMMAND_ERR, "id").as_str()),
            Response::Err(("id", None))
        );
        assert_eq!(
            Response::from(format!("{} {} {}", COMMAND_ERR, "id", "a description").as_str()),
            Response::Err(("id", Some("a description")))
        );

        assert_eq!(
            Response::from(COMMAND_S),
            Response::Custom((COMMAND_S, None))
        );
        assert_eq!(
            Response::from(format!("{} {}", COMMAND_S, "keyword").as_str()),
            Response::Custom((COMMAND_S, Some("keyword")))
        );
        assert_eq!(
            Response::from(
                format!("{} {} {}", COMMAND_S, "keyword", "status information").as_str()
            ),
            Response::S(("keyword", "status information"))
        );

        assert_eq!(
            Response::from(COMMAND_INQUIRE),
            Response::Custom((COMMAND_INQUIRE, None)),
        );
        assert_eq!(
            Response::from(format!("{} {}", COMMAND_INQUIRE, "keyword").as_str()),
            Response::Custom((COMMAND_INQUIRE, Some("keyword")))
        );
        assert_eq!(
            Response::from(format!("{} {} {}", COMMAND_INQUIRE, "keyword", "params").as_str()),
            Response::Inquire(("keyword", "params"))
        );

        assert_eq!(
            Response::from(COMMAND_D),
            Response::Custom((COMMAND_D, None)),
        );
        assert_eq!(
            Response::from(format!("{} {}", COMMAND_D, "some data").as_str()),
            Response::D("some data"),
        );

        assert_eq!(Response::from(COMMAND_COMMENT), Response::Comment(None),);
        assert_eq!(
            Response::from(format!("{} {}", COMMAND_COMMENT, "comment data").as_str()),
            Response::Comment(Some("comment data")),
        );
    }
}
