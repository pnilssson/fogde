use std::time::Duration;

use herdr_client::{Client, Error, Event, Scrollback, Subscription};
use protocol::rpc::{Failure, Message, SERVER_ERROR};
use protocol::terminals::{ScrolledEvent, SeekTerminal, SeekTerminalParams, TerminalScrolled};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::rpc::{Handler, failure, notification};
use crate::session::Session;

const RETRY: Duration = Duration::from_secs(1);

impl Handler for SeekTerminal {
    const ORDERED: bool = true;

    async fn handle(session: &Session, params: SeekTerminalParams) -> Result<(), Failure> {
        let client = session.herdr().await?.client();
        let pane = client
            .panes()
            .await
            .map_err(failure)?
            .into_iter()
            .find(|pane| pane.workspace == params.id)
            .ok_or_else(|| Failure::new(SERVER_ERROR, "the terminal is gone"))?;

        client
            .scroll_pane(&pane.id, u64::from(params.offset))
            .await
            .map_err(failure)
    }
}

pub async fn follow(
    client: Client,
    pane: String,
    id: String,
    outgoing: mpsc::UnboundedSender<Message>,
    watching: JoinHandle<()>,
) {
    let keep = async {
        loop {
            if let Err(error) = listen(&client, &pane, &id, &outgoing).await {
                tracing::warn!(%error, %pane, "lost the terminal's scroll position");
            }

            tokio::time::sleep(RETRY).await;
        }
    };

    tokio::select! {
        () = keep => {}
        _ = watching => {}
    }
}

async fn listen(
    client: &Client,
    pane: &str,
    id: &str,
    outgoing: &mpsc::UnboundedSender<Message>,
) -> Result<(), Error> {
    let mut events = client
        .subscribe(&[Subscription::ScrollChanged {
            pane_id: pane.to_owned(),
        }])
        .await?;
    if let Some(scrollback) = client
        .panes()
        .await?
        .into_iter()
        .find(|found| found.id == pane)
        .and_then(|found| found.scrollback)
    {
        send(outgoing, id, scrollback);
    }

    loop {
        if let Event::Scrolled {
            pane: changed,
            scrollback,
        } = events.event().await?
            && changed == pane
        {
            send(outgoing, id, scrollback);
        }
    }
}

fn send(outgoing: &mpsc::UnboundedSender<Message>, id: &str, scrollback: Scrollback) {
    if let Some(message) = notification::<ScrolledEvent>(TerminalScrolled {
        id: id.to_owned(),
        offset: clamp(scrollback.offset),
        max: clamp(scrollback.max),
        rows: clamp(scrollback.rows),
    }) {
        let _ = outgoing.send(message);
    }
}

fn clamp(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
