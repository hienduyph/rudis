use tokio::sync::broadcast;

#[derive(Debug)]
pub(crate) struct Shutdown {
    shutdown: bool,
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    pub(crate) fn new(notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown {
            shutdown: false,
            notify,
        }
    }

    pub(crate) fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    pub(crate) async fn recv(&mut self) {
        if self.shutdown {
            return;
        }

        let _ = self.notify.recv().await;
        self.shutdown = true;
    }
}
