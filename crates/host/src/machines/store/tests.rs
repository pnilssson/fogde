use super::*;
use crate::db;

fn vps(id: &str, target: &str) -> Server {
    Server {
        id: id.to_owned(),
        target: target.to_owned(),
        triple: "x86_64-unknown-linux-musl".to_owned(),
    }
}

fn id(id: &str) -> MachineId {
    MachineId::Server { id: id.to_owned() }
}

fn connected(machines: &[Machine]) -> Vec<bool> {
    machines.iter().map(|machine| machine.connected).collect()
}

#[tokio::test]
async fn lists_the_computer_first_and_once() {
    let pool = db::memory().await;

    add_server(&pool, &vps("a1", "my-vps")).await.unwrap();
    add_computer(&pool).await.unwrap();
    add_computer(&pool).await.unwrap();

    assert_eq!(
        list(&pool).await.unwrap(),
        [
            Machine {
                server: None,
                name: None,
                connected: true,
            },
            Machine {
                server: Some(vps("a1", "my-vps")),
                name: None,
                connected: true,
            },
        ]
    );
}

#[tokio::test]
async fn keeps_one_entry_per_server() {
    let pool = db::memory().await;

    add_server(&pool, &vps("a1", "my-vps")).await.unwrap();
    add_server(&pool, &vps("b2", "work")).await.unwrap();
    add_server(&pool, &vps("a1", "root@1.2.3.4")).await.unwrap();
    add_server(&pool, &vps("c3", "work")).await.unwrap();

    let servers: Vec<_> = list(&pool)
        .await
        .unwrap()
        .into_iter()
        .filter_map(|machine| machine.server)
        .collect();
    assert_eq!(servers, [vps("a1", "root@1.2.3.4"), vps("c3", "work")]);
}

#[tokio::test]
async fn remembers_a_disconnected_server() {
    let pool = db::memory().await;
    add_computer(&pool).await.unwrap();
    add_server(&pool, &vps("a1", "my-vps")).await.unwrap();

    set_connected(&pool, &id("a1"), false).await.unwrap();

    assert_eq!(connected(&list(&pool).await.unwrap()), [true, false]);
    assert_eq!(
        server(&pool, "a1").await.unwrap(),
        Some(Machine {
            server: Some(vps("a1", "my-vps")),
            name: None,
            connected: false,
        })
    );

    set_connected(&pool, &id("a1"), true).await.unwrap();

    assert_eq!(connected(&list(&pool).await.unwrap()), [true, true]);
}

#[tokio::test]
async fn keeps_one_machine_connected() {
    let pool = db::memory().await;
    add_computer(&pool).await.unwrap();
    add_server(&pool, &vps("a1", "my-vps")).await.unwrap();
    set_connected(&pool, &id("a1"), false).await.unwrap();

    assert!(matches!(
        set_connected(&pool, &MachineId::Computer, false).await,
        Err(Error::Last)
    ));
    assert!(matches!(
        remove(&pool, &MachineId::Computer).await,
        Err(Error::Last)
    ));
    assert_eq!(connected(&list(&pool).await.unwrap()), [true, false]);

    remove(&pool, &id("a1")).await.unwrap();

    assert_eq!(connected(&list(&pool).await.unwrap()), [true]);
}

#[tokio::test]
async fn adding_the_computer_again_connects_it() {
    let pool = db::memory().await;
    add_computer(&pool).await.unwrap();
    add_server(&pool, &vps("a1", "my-vps")).await.unwrap();
    set_connected(&pool, &MachineId::Computer, false)
        .await
        .unwrap();

    add_computer(&pool).await.unwrap();

    assert_eq!(connected(&list(&pool).await.unwrap()), [true, true]);
}

#[tokio::test]
async fn keeps_a_name_until_it_is_cleared() {
    let pool = db::memory().await;
    add_computer(&pool).await.unwrap();
    add_server(&pool, &vps("a1", "my-vps")).await.unwrap();
    let names = async || -> Vec<Option<String>> {
        list(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|machine| machine.name)
            .collect()
    };

    rename(&pool, &MachineId::Computer, " Desktop ")
        .await
        .unwrap();
    rename(&pool, &id("a1"), "Test VPS").await.unwrap();
    add_server(&pool, &vps("a1", "root@1.2.3.4")).await.unwrap();

    assert_eq!(
        names().await,
        [Some("Desktop".to_owned()), Some("Test VPS".to_owned())]
    );

    rename(&pool, &id("a1"), "  ").await.unwrap();

    assert_eq!(names().await, [Some("Desktop".to_owned()), None]);
    assert!(matches!(
        rename(&pool, &id("b2"), "Work").await,
        Err(Error::Missing)
    ));
}

#[tokio::test]
async fn refuses_to_connect_a_removed_machine() {
    let pool = db::memory().await;

    assert!(matches!(
        set_connected(&pool, &id("a1"), true).await,
        Err(Error::Missing)
    ));
}
