pub trait ServerMonitor {
    fn tick(&mut self);
    fn reserved(&mut self);
    fn connected(&mut self);
    fn disconnected(&mut self);
    fn message_received(&mut self);
    fn message_sent(&mut self);
    fn message_acknowledged(&mut self);
}

pub trait ClientMonitor {
    fn tick(&mut self);
    fn connecting(&mut self);
    fn connected(&mut self);
    fn disconnected(&mut self);
    fn message_received(&mut self);
    fn message_sent(&mut self);
    fn message_acknowledged(&mut self);
}

pub struct EmptyClientMonitor;

impl EmptyClientMonitor {
    pub fn new() -> Self {
        Self
    }
}

impl ClientMonitor for EmptyClientMonitor {
    fn tick(&mut self) { }
    fn connecting(&mut self) { }
    fn connected(&mut self) { }
    fn disconnected(&mut self) { }
    fn message_received(&mut self) { }
    fn message_sent(&mut self) { }
    fn message_acknowledged(&mut self) { }
}
