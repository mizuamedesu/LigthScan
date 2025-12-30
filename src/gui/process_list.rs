use crate::platform::ProcessInfo;
use crate::scanner::Process;
use eframe::egui;

/// UI component for displaying and selecting processes
#[derive(Default)]
pub struct ProcessListView {
    processes: Vec<ProcessInfo>,
    filter: String,
    last_refresh: Option<std::time::Instant>,
}

impl ProcessListView {
    /// Refreshes the process list
    fn refresh(&mut self) {
        match Process::list_all() {
            Ok(mut processes) => {
                // Sort by name
                processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                self.processes = processes;
                self.last_refresh = Some(std::time::Instant::now());
            }
            Err(e) => {
                tracing::error!("Failed to refresh process list: {}", e);
            }
        }
    }

    /// Renders the process list UI
    /// Returns Some(ProcessInfo) if a process was selected
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<ProcessInfo> {
        let mut selected = None;

        // Auto-refresh on first display or if button clicked
        if self.last_refresh.is_none() {
            self.refresh();
        }

        // Refresh button and filter
        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() {
                self.refresh();
            }

            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter);
        });

        ui.separator();

        // Status
        if let Some(last_refresh) = self.last_refresh {
            let elapsed = last_refresh.elapsed().as_secs();
            ui.label(format!(
                "Showing {} processes (refreshed {}s ago)",
                self.processes.len(),
                elapsed
            ));
        }

        ui.separator();

        // Process list (scrollable)
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                // Table header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("PID").strong().size(12.0));
                    ui.add_space(60.0);
                    ui.label(egui::RichText::new("Process Name").strong().size(12.0));
                });

                ui.separator();

                // Filter processes
                let filter_lower = self.filter.to_lowercase();
                let filtered: Vec<&ProcessInfo> = self
                    .processes
                    .iter()
                    .filter(|p| {
                        filter_lower.is_empty()
                            || p.name.to_lowercase().contains(&filter_lower)
                            || p.pid.to_string().contains(&filter_lower)
                    })
                    .collect();

                // Display processes
                for process in filtered {
                    ui.horizontal(|ui| {
                        ui.label(format!("{:<8}", process.pid));
                        if ui.button(&process.name).clicked() {
                            selected = Some(process.clone());
                        }
                    });
                }

                if self.processes.is_empty() {
                    ui.label("No processes found");
                }
            });

        selected
    }
}
