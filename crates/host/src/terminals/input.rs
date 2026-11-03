use std::future;

use protocol::rpc::Failure;
use protocol::terminals::{
    DisplayTerminal, DisplayTerminalParams, FocusTerminal, FocusTerminalParams, PasteFile,
    PasteFileParams, ResizeTerminal, ResizeTerminalParams, TerminalInput, TerminalInputParams,
    TerminalReply,
};

use super::files;
use super::stream::Control;
use crate::rpc::{Handler, failure};
use crate::session::Session;

impl Handler for TerminalInput {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: TerminalInputParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Input(params.data.into_bytes())),
        )
    }
}

impl Handler for TerminalReply {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: TerminalInputParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Reply(params.data.into_bytes())),
        )
    }
}

impl Handler for FocusTerminal {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: FocusTerminalParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Focus(params.focused)),
        )
    }
}

impl Handler for ResizeTerminal {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: ResizeTerminalParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Resize(params.size)),
        )
    }
}

impl Handler for DisplayTerminal {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: DisplayTerminalParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Display(params.display)),
        )
    }
}

impl Handler for PasteFile {
    const ORDERED: bool = true;

    async fn handle(session: &Session, params: PasteFileParams) -> Result<(), Failure> {
        let path = files::save(&session.shared.files(), &params.name, params.data)
            .await
            .map_err(failure)?;
        let paste = files::paste(&path);

        session
            .terminals
            .send(&params.id, Control::Input(paste.into_bytes()))
    }
}
