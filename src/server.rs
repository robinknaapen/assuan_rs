use crate::{
    errors,
    request::Request,
    response::{Response, ResponseErr},
};

use async_std::{
    io::{Error, Write},
    prelude::*,
};

#[derive(Debug)]
pub enum ServerError {
    Write(Error),
}

pub type HandlerRequest<'a> = (&'a str, Option<&'a str>);
pub type HandlerResult = Result<Option<Response>, (ResponseErr, Option<String>)>;

pub type OptionRequest<'a> = (&'a str, Option<&'a str>);
pub type OptionResult = Result<Response, (ResponseErr, Option<String>)>;

pub type HelpResult = Option<Vec<String>>;

pub trait Handler {
    // handle handles custom requests
    fn handle(&mut self, request: HandlerRequest) -> impl Future<Output = HandlerResult>;

    // option is called when an option is requested
    fn option(&mut self, option: OptionRequest) -> impl Future<Output = OptionResult>;

    // return a list of custom commands if any
    fn help(&mut self) -> HelpResult;

    // reset can be a noop
    fn reset(&mut self);
}

pub async fn start<S, W, H>(mut r: S, mut w: W, mut handler: H) -> Result<(), ServerError>
where
    S: Stream<Item = Result<String, std::io::Error>> + Unpin,
    W: Write + Unpin,
    H: Handler,
{
    writeln!(
        w,
        "{}",
        Response::Ok(Some(String::from("Pleased to meet you")))
    )
    .await
    .unwrap();

    while let Some(line) = r.next().await {
        match line {
            Err(e) => {
                let wr = writeln!(
                    w,
                    "{}",
                    Response::Err((
                        ResponseErr::Gpg(errors::GpgErrorCode::Unexpected),
                        Some(e.to_string())
                    ))
                )
                .await;

                if let Err(err) = wr {
                    return Err(ServerError::Write(err));
                };
            }
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if line.len() > 1000 {
                    let wr = writeln!(
                        w,
                        "{}",
                        Response::Err((ResponseErr::Gpg(errors::GpgErrorCode::TooLarge), None))
                    )
                    .await;
                    if let Err(err) = wr {
                        return Err(ServerError::Write(err));
                    };

                    continue;
                }

                let request = Request::from(line);
                let wr = match request {
                    Request::Comment(_) => continue,

                    Request::Reset => {
                        handler.reset();
                        writeln!(w, "{}", Response::Ok(None)).await
                    }

                    Request::Bye => writeln!(w, "{}", Response::Ok(None)).await,
                    Request::Nop => writeln!(w, "{}", Response::Ok(None)).await,

                    Request::Option((s, None)) => match handler.option((s.as_ref(), None)).await {
                        Ok(response) => writeln!(w, "{}", response).await,
                        Err(e) => writeln!(w, "{}", Response::Err(e)).await,
                    },

                    Request::Option((s, Some(v))) => {
                        match handler.option((s.as_ref(), Some(v.as_ref()))).await {
                            Ok(response) => writeln!(w, "{}", response).await,
                            Err(e) => writeln!(w, "{}", Response::Err(e)).await,
                        }
                    }

                    Request::Unknown((v, None)) => match handler.handle((v.as_ref(), None)).await {
                        Ok(None) => return Ok(()),
                        Ok(Some(response)) => writeln!(w, "{}", response).await,
                        Err(e) => writeln!(w, "{}", Response::Err(e)).await,
                    },

                    Request::Unknown((v, Some(o))) => {
                        match handler.handle((v.as_ref(), Some(o.as_ref()))).await {
                            Ok(None) => return Ok(()),
                            Ok(Some(response)) => writeln!(w, "{}", response).await,
                            Err(e) => writeln!(w, "{}", Response::Err(e)).await,
                        }
                    }
                    Request::D(_) => todo!(),
                    Request::End => todo!(),
                    Request::Help => {
                        if let Some(v) = handler.help() {
                            for s in v {
                                let _ = writeln!(w, "{}", Response::Comment(Some(s))).await;
                            }
                        }
                        writeln!(w, "{}", Response::Ok(None)).await
                    }
                    Request::Cancel => todo!(),

                    Request::Quit => {
                        break;
                    }
                };

                if let Err(err) = wr {
                    return Err(ServerError::Write(err));
                };
            }
        }
    }

    Ok(())
}
