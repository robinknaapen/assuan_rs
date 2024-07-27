use strum::{AsRefStr, Display, EnumString};

#[derive(Clone, PartialEq, Debug, EnumString, Display, AsRefStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Command {
    Bye,
    Reset,
    End,
    Help,
    Quit,
    Option,
    Cancel,
    Nop,
    Ok,
    Err,
    S,
    Inquire,
    D,

    #[strum(serialize = "#")]
    Comment,
}
