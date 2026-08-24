use std::time::SystemTime;

use crate::models::event::Event;
use crate::models::severity::Severity;

#[derive(Debug, Clone)]
pub struct Alert {
    pub rule_name: String,
    pub description: String,
    pub events: Vec<Event>,
    pub severity: Severity,
    pub detected_at: SystemTime
}