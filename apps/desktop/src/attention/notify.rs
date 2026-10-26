use tokio::sync::oneshot;

#[cfg(target_os = "linux")]
fn wait(handle: notify_rust::NotificationHandle, clicked: oneshot::Sender<()>) {
    handle.wait_for_action(|action| {
        if action == "default" {
            let _ = clicked.send(());
        }
    });
}

#[cfg(not(target_os = "linux"))]
fn wait(handle: notify_rust::NotificationHandle, clicked: oneshot::Sender<()>) {
    let result = handle.wait_for_response(|response: &notify_rust::NotificationResponse| {
        if matches!(
            response,
            notify_rust::NotificationResponse::Default
                | notify_rust::NotificationResponse::Action(_)
        ) {
            let _ = clicked.send(());
        }
    });
    if let Err(error) = result {
        tracing::warn!(%error, "could not follow a notification");
    }
}

pub fn show(title: String, body: String) -> oneshot::Receiver<()> {
    let (clicked, receiver) = oneshot::channel();
    std::thread::spawn(move || {
        let mut notification = notify_rust::Notification::new();
        notification.summary(&title).body(&body);
        #[cfg(target_os = "linux")]
        notification.action("default", "Show").appname("Fogde");
        #[cfg(windows)]
        if !cfg!(debug_assertions) {
            notification.app_id("dev.fogde.app");
        }
        match notification.show() {
            Ok(handle) => wait(handle, clicked),
            Err(error) => tracing::warn!(%error, "could not show a notification"),
        }
    });
    receiver
}
