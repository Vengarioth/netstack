pub trait ServerMonitor {
    fn tick(&mut self);
    fn connected(&mut self);
    fn disconnected(&mut self);
    fn message(&mut self);
}
