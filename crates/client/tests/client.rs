mod server;

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use client::{Client, Error, State};
use protocol::Method;
use protocol::rpc::{Failure, OFFLINE, SERVER_ERROR};
use protocol::terminals::FrameEvent;
use serde_json::json;
use server::Server;
use tokio::time::timeout;

struct Name;

impl Method for Name {
    const NAME: &'static str = "test.name";
    type Params = ();
    type Result = String;
}

struct Hold;

impl Method for Hold {
    const NAME: &'static str = "test.hold";
    type Params = ();
    type Result = ();
}

async fn pair() -> std::io::Result<(Arc<Server>, Arc<Server>, Client, Client)> {
    let local = Server::start("local").await?;
    let remote = Server::start("remote").await?;
    let own = Client::local(local.connection());
    let server = Client::local(remote.connection());

    Ok((local, remote, own, server))
}

#[tokio::test]
async fn an_unavailable_server_settles_without_retrying_or_blocking_this_computer() {
    let (_local, remote, own, server) = pair().await.unwrap();
    remote.options.lock().unwrap().refuse = 1;

    assert_eq!(server.call::<Name>(()).await, Err(Error::Unreachable));
    assert!(matches!(server.state(), State::Disconnected { .. }));
    assert_eq!(own.call::<Name>(()).await.unwrap(), "local");
    assert_eq!(remote.sockets.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_pending_server_handshake_does_not_block_a_local_request() {
    let (_local, remote, own, server) = pair().await.unwrap();
    remote.options.lock().unwrap().stall = true;

    let waiting = tokio::spawn({
        let server = server.clone();
        async move { server.call::<Name>(()).await }
    });

    assert_eq!(own.call::<Name>(()).await.unwrap(), "local");
    assert_eq!(server.state(), State::Connecting);
    assert_eq!(own.state(), State::Connected);
    waiting.abort();
}

#[tokio::test]
async fn reconnect_replaces_only_the_disconnected_machines_socket() {
    let (local, remote, own, server) = pair().await.unwrap();
    remote.options.lock().unwrap().refuse = 1;
    own.call::<Name>(()).await.unwrap();
    assert!(server.call::<Name>(()).await.is_err());

    server.reconnect().await.unwrap();

    assert_eq!(server.call::<Name>(()).await.unwrap(), "remote");
    assert_eq!(own.state(), State::Connected);
    assert_eq!(local.sockets.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn events_never_cross_from_one_machines_connection_to_another() {
    let (_local, remote, own, server) = pair().await.unwrap();
    let mut own_frames = own.listen::<FrameEvent>();
    let mut server_frames = server.listen::<FrameEvent>();
    own.call::<Name>(()).await.unwrap();
    server.call::<Name>(()).await.unwrap();
    let frame = json!({"id": "w1", "cols": 80, "rows": 24, "full": true, "data": "aGk="});

    remote.push("terminals.frame", &frame);
    let received = timeout(Duration::from_secs(5), server_frames.recv())
        .await
        .unwrap()
        .unwrap();
    server.reset();
    remote.push("terminals.frame", &frame);
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(received.id, "w1");
    assert!(server_frames.try_recv().is_none());
    assert!(own_frames.try_recv().is_none());
}

#[tokio::test]
async fn a_host_error_is_reported_without_marking_the_machine_offline() {
    let (_local, remote, _own, server) = pair().await.unwrap();
    remote.options.lock().unwrap().error = Some("could not start the terminal service".to_owned());

    let error = server.call::<Name>(()).await.unwrap_err();

    assert_eq!(error.to_string(), "could not start the terminal service");
    assert!(!error.retryable());
    assert_eq!(server.state(), State::Connected);
    assert_eq!(remote.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn reconnect_rejects_old_requests_without_waiting_for_the_old_socket() {
    let (_local, remote, _own, server) = pair().await.unwrap();
    server.call::<Name>(()).await.unwrap();
    remote.options.lock().unwrap().hold = true;
    let outcome = tokio::spawn({
        let server = server.clone();
        async move { server.call::<Hold>(()).await }
    });
    while remote.requests.lock().unwrap().len() < 2 {
        tokio::task::yield_now().await;
    }

    server.reconnect().await.unwrap();

    let outcome = timeout(Duration::from_millis(50), outcome)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(outcome, Err(Error::Lost));
}

#[tokio::test]
async fn a_server_that_cannot_be_reached_is_retried_when_the_user_returns() {
    let failure = Failure::new(OFFLINE, "could not reach my-vps");
    let unreachable = Client::new(move || std::future::ready(Err(Error::Rpc(failure.clone()))));

    assert!(unreachable.call::<Name>(()).await.is_err());
    assert_eq!(
        unreachable.state(),
        State::Disconnected {
            error: "could not reach my-vps".to_owned(),
            retryable: true,
        }
    );
}

#[tokio::test]
async fn a_rejected_key_waits_for_the_user_instead_of_retrying() {
    let failure = Failure::new(SERVER_ERROR, "my-vps did not accept your SSH key");
    let rejected = Client::new(move || std::future::ready(Err(Error::Rpc(failure.clone()))));

    assert!(rejected.call::<Name>(()).await.is_err());
    assert!(matches!(
        rejected.state(),
        State::Disconnected {
            retryable: false,
            ..
        }
    ));
}

#[tokio::test]
async fn a_lost_connection_is_retried_when_the_user_returns() {
    let (_local, remote, _own, server) = pair().await.unwrap();
    server.call::<Name>(()).await.unwrap();

    remote.drop_sockets();

    let lost = State::Disconnected {
        error: "The connection was lost.".to_owned(),
        retryable: true,
    };
    let mut states = server.watch();
    timeout(
        Duration::from_secs(5),
        states.wait_for(|state| *state == lost),
    )
    .await
    .unwrap()
    .unwrap();
}

#[tokio::test]
async fn a_server_connects_through_its_own_host_first() {
    let (local, _remote, own, _server) = pair().await.unwrap();
    let server = Client::server(own, local.connection(), "vps 1".to_owned());

    assert_eq!(server.call::<Name>(()).await.unwrap(), "local");
    assert_eq!(
        local.requests.lock().unwrap().as_slice(),
        ["server.connect", "test.name"]
    );
}

#[tokio::test]
async fn a_listener_that_stops_reading_does_not_hold_up_replies() {
    let (_local, remote, _own, server) = pair().await.unwrap();
    let _idle = server.listen::<FrameEvent>();
    server.call::<Name>(()).await.unwrap();
    let frame = json!({"id": "w1", "cols": 80, "rows": 24, "full": true, "data": "aGk="});

    for _ in 0..300 {
        remote.push("terminals.frame", &frame);
    }
    let reply = timeout(Duration::from_secs(5), server.call::<Name>(())).await;

    assert_eq!(reply.unwrap().unwrap(), "remote");
}
