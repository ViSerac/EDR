use crate::models::event::Event;
use crate::models::alert::Alert;

pub trait Rule {
    fn process(&mut self, event: &Event) -> Option<Alert>;
}