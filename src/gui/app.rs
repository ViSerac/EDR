use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use eframe::egui::{self, Align, Color32, Layout, RichText};

use crate::gui::format::{
    category_color, describe_event, format_time, severity_color, severity_label, EventCategory,
    EventRow,
};
use crate::models::alert::Alert;
use crate::models::severity::Severity;
use crate::pipeline::spawn_capture_thread;

const MAX_EVENTS: usize = 10_000;
const MAX_ALERTS: usize = 1_000;

pub struct EdrApp {
    event_rx: Receiver<crate::models::event::Event>,
    alert_rx: Receiver<Alert>,

    events: Vec<EventRow>,
    /// Alerts paired with a stable sequence number (used as widget id salt,
    /// so collapsing state survives newer alerts being inserted).
    alerts: Vec<(u64, Alert)>,
    total_events: u64,
    alert_seq: u64,

    search: String,
    show_process: bool,
    show_network: bool,
    show_file: bool,

    show_alerts_panel: bool,
    show_about: bool,
}

impl EdrApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> EdrApp {
        cc.egui_ctx.set_theme(egui::ThemePreference::Dark);

        let (event_tx, event_rx) = mpsc::channel();
        let (alert_tx, alert_rx) = mpsc::channel();

        spawn_capture_thread(event_tx, alert_tx, cc.egui_ctx.clone());

        EdrApp {
            event_rx,
            alert_rx,
            events: Vec::new(),
            alerts: Vec::new(),
            total_events: 0,
            alert_seq: 0,
            search: String::new(),
            show_process: true,
            show_network: true,
            show_file: true,
            show_alerts_panel: true,
            show_about: false,
        }
    }

    fn drain_channels(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.total_events += 1;
            self.events.push(EventRow::from_event(&event));
        }
        if self.events.len() > MAX_EVENTS {
            let excess = self.events.len() - MAX_EVENTS;
            self.events.drain(0..excess);
        }

        while let Ok(alert) = self.alert_rx.try_recv() {
            self.alert_seq += 1;
            self.alerts.push((self.alert_seq, alert));
        }
        if self.alerts.len() > MAX_ALERTS {
            let excess = self.alerts.len() - MAX_ALERTS;
            self.alerts.drain(0..excess);
        }
    }

    fn row_passes_filters(&self, row: &EventRow, query: &str) -> bool {
        let category_visible = match row.category {
            EventCategory::Process => self.show_process,
            EventCategory::Network => self.show_network,
            EventCategory::File => self.show_file,
        };
        category_visible && (query.is_empty() || row.search_text.contains(query))
    }

    fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Clear events").clicked() {
                        self.events.clear();
                        ui.close_menu();
                    }
                    if ui.button("Clear alerts").clicked() {
                        self.alerts.clear();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_alerts_panel, "Alerts panel");
                    ui.separator();
                    ui.checkbox(&mut self.show_process, "Process events");
                    ui.checkbox(&mut self.show_network, "Network events");
                    ui.checkbox(&mut self.show_file, "File events");
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Filters:").weak());
                ui.toggle_value(&mut self.show_process, "Process");
                ui.toggle_value(&mut self.show_network, "Network");
                ui.toggle_value(&mut self.show_file, "File");

                ui.separator();

                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text("Search events...")
                        .desired_width(220.0),
                );
                if !self.search.is_empty() && ui.small_button("Clear").clicked() {
                    self.search.clear();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let alert_count_text = format!("Alerts: {}", self.alerts.len());
                    let max_severity: Option<Severity> =
                        self.alerts.iter().map(|(_, alert)| alert.severity).max();
                    match max_severity {
                        Some(severity) => ui.label(
                            RichText::new(alert_count_text)
                                .color(severity_color(severity))
                                .strong(),
                        ),
                        None => ui.label(RichText::new(alert_count_text).weak()),
                    };

                    ui.separator();
                    ui.label(RichText::new(format!("Events: {}", self.total_events)).weak());
                    ui.separator();
                    ui.label(RichText::new("Monitoring").color(Color32::from_rgb(74, 222, 128)));
                });
            });
            ui.add_space(4.0);
        });
    }

    fn alerts_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("alerts_panel")
            .default_width(360.0)
            .min_width(280.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.heading("Alerts");
                    ui.label(RichText::new(format!("({})", self.alerts.len())).weak());
                });
                ui.separator();

                if self.alerts.is_empty() {
                    ui.add_space(12.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("No alerts detected yet.").weak());
                    });
                    return;
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (seq, alert) in self.alerts.iter().rev() {
                            alert_card(ui, alert, *seq);
                            ui.add_space(6.0);
                        }
                    });
            });
    }

    fn event_feed(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let query = self.search.to_lowercase();
            let filtered: Vec<usize> = self
                .events
                .iter()
                .enumerate()
                .filter(|(_, row)| self.row_passes_filters(row, &query))
                .map(|(index, _)| index)
                .collect();

            if filtered.is_empty() {
                ui.add_space(24.0);
                ui.vertical_centered(|ui| {
                    let message = if self.events.is_empty() {
                        "Waiting for events..."
                    } else {
                        "No events match the current filters."
                    };
                    ui.label(RichText::new(message).weak());
                });
                return;
            }

            let row_height = ui.text_style_height(&egui::TextStyle::Body);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, row_height, filtered.len(), |ui, range| {
                    for display_index in range {
                        // Newest first: walk the filtered list back to front.
                        let event_index = filtered[filtered.len() - 1 - display_index];
                        let row = &self.events[event_index];

                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&row.time).weak().monospace());
                            ui.label(
                                RichText::new(format!("{:<4}", row.tag))
                                    .color(category_color(row.category))
                                    .monospace()
                                    .strong(),
                            );
                            ui.add(egui::Label::new(&row.description).truncate());
                        });
                    }
                });
        });
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }

        egui::Window::new("About")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut self.show_about)
            .show(ctx, |ui| {
                ui.heading("mini EDR");
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.add_space(6.0);
                ui.label("Educational Endpoint Detection & Response for Windows.");
                ui.label("Monitors processes, TCP/UDP endpoints and file activity,");
                ui.label("running rule-based detection on every event.");
            });
    }
}

impl eframe::App for EdrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_channels();

        // Safety net: even without repaint requests from the capture thread,
        // keep polling the channels at least once a second.
        ctx.request_repaint_after(Duration::from_secs(1));

        self.menu_bar(ctx);
        self.status_bar(ctx);
        if self.show_alerts_panel {
            self.alerts_panel(ctx);
        }
        self.event_feed(ctx);
        self.about_window(ctx);
    }
}

fn alert_card(ui: &mut egui::Ui, alert: &Alert, seq: u64) {
    let color = severity_color(alert.severity);

    egui::Frame::group(ui.style())
        .fill(color.gamma_multiply(0.08))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.6)))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(severity_label(alert.severity))
                        .color(color)
                        .monospace()
                        .strong(),
                );
                ui.label(RichText::new(&alert.rule_name).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(format_time(alert.detected_at)).weak().monospace());
                });
            });

            ui.label(&alert.description);

            if !alert.events.is_empty() {
                egui::CollapsingHeader::new(
                    RichText::new(format!("{} related event(s)", alert.events.len())).small(),
                )
                .id_salt(seq)
                .show(ui, |ui| {
                    for event in &alert.events {
                        ui.label(RichText::new(describe_event(event)).weak().small());
                    }
                });
            }
        });
}
