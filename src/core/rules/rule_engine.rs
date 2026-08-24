use crate::core::rules::rule_trait::Rule;
use crate::models::alert::Alert;
use crate::models::event::Event;

pub struct RuleEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleEngine {
    pub fn new(rules: Vec<Box<dyn Rule>>) -> RuleEngine {
        RuleEngine { rules }
    }

    pub fn process_event(&mut self, event: &Event) -> Vec<Alert> {
        let mut alerts = Vec::new();

        for rule in &mut self.rules {
            if let Some(alert) = rule.process(event) {
                alerts.push(alert);
            }
        }

        alerts
    }
}