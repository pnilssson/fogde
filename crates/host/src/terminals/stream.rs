use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use protocol::rpc::Message;
use protocol::terminals::{
    EndedEvent, FrameEvent, Scroll, ScrollTerminalParams, Size, TerminalEnded, TerminalFrame,
};
use tokio::sync::mpsc;

use crate::rpc::notify;

pub enum Control {
    Input(Vec<u8>),
    Resize(Size),
    Scroll(ScrollTerminalParams),
}

pub async fn watch(
    mut stream: herdr_client::Terminal,
    id: String,
    mut controls: mpsc::UnboundedReceiver<Control>,
    outgoing: mpsc::UnboundedSender<Message>,
) {
    let error = loop {
        tokio::select! {
            frame = stream.next() => match frame {
                Ok(frame) => notify::<FrameEvent>(&outgoing, TerminalFrame {
                    id: id.clone(),
                    cols: frame.width,
                    rows: frame.height,
                    full: frame.full,
                    data: STANDARD.encode(frame.bytes),
                }),
                Err(error) => break error,
            },
            control = controls.recv() => {
                let sent = match control {
                    Some(Control::Input(bytes)) => stream.input(&bytes).await,
                    Some(Control::Resize(size)) => stream.resize(size.cols, size.rows).await,
                    Some(Control::Scroll(scroll)) => {
                        let direction = match scroll.direction {
                            Scroll::Up => herdr_client::Scroll::Up,
                            Scroll::Down => herdr_client::Scroll::Down,
                        };
                        stream.scroll(direction, scroll.lines, scroll.column, scroll.row).await
                    }
                    None => return,
                };
                if let Err(error) = sent {
                    break error;
                }
            }
        }
    };

    tracing::info!(%error, %id, "terminal stream ended");
    notify::<EndedEvent>(
        &outgoing,
        TerminalEnded {
            id,
            error: match error {
                herdr_client::Error::Closed => None,
                error => Some(error.to_string()),
            },
        },
    );
}
