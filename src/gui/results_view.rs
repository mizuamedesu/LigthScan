use crate::scanner::Scanner;
use crate::types::{ScanResult, ValueType};
use eframe::egui;
use std::sync::{Arc, Mutex};

/// UI component for displaying scan results
pub struct ResultsView {
    results: Vec<ScanResult>,
    value_type: ValueType,
    page_offset: usize,
    page_size: usize,
    edit_address: Option<usize>,
    edit_value: String,
}

impl Default for ResultsView {
    fn default() -> Self {
        Self {
            results: Vec::new(),
            value_type: ValueType::I32,
            page_offset: 0,
            page_size: 100,
            edit_address: None,
            edit_value: String::new(),
        }
    }
}

impl ResultsView {
    pub fn clear(&mut self) {
        self.results.clear();
        self.page_offset = 0;
        self.edit_address = None;
        self.edit_value.clear();
    }

    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    pub fn update_from_scanner(&mut self, scanner: &Scanner) {
        self.results = scanner.results().results.clone();
        self.value_type = scanner.results().value_type;
        self.page_offset = 0;
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, scanner: &Option<Arc<Mutex<Scanner>>>) {
        if self.results.is_empty() {
            ui.label("No results. Perform a scan to see results here.");
            return;
        }

        // Pagination controls
        ui.horizontal(|ui| {
            ui.label(format!("Total results: {}", self.results.len()));

            ui.separator();

            if ui.button("◀ Prev").clicked() && self.page_offset > 0 {
                self.page_offset = self.page_offset.saturating_sub(self.page_size);
            }

            let current_page = (self.page_offset / self.page_size) + 1;
            let total_pages = (self.results.len() + self.page_size - 1) / self.page_size;
            ui.label(format!("Page {}/{}", current_page, total_pages));

            if ui.button("Next ▶").clicked() && self.page_offset + self.page_size < self.results.len() {
                self.page_offset += self.page_size;
            }

            ui.separator();

            ui.label("Per page:");
            ui.add(egui::DragValue::new(&mut self.page_size).range(10..=1000));
        });

        ui.separator();

        // Results table
        egui::ScrollArea::vertical()
            .max_height(500.0)
            .show(ui, |ui| {
                egui::Grid::new("results_grid")
                    .striped(true)
                    .num_columns(4)
                    .show(ui, |ui| {
                        // Header
                        ui.label(egui::RichText::new("Address").strong());
                        ui.label(egui::RichText::new("Value").strong());
                        ui.label(egui::RichText::new("Previous").strong());
                        ui.label(egui::RichText::new("Actions").strong());
                        ui.end_row();

                        // Get current page of results
                        let end = (self.page_offset + self.page_size).min(self.results.len());
                        let page_results = &self.results[self.page_offset..end];

                        // Display results
                        for result in page_results {
                            // Address
                            ui.label(format!("0x{:X}", result.address));

                            // Current value
                            if let Some(value) = result.parse_value(self.value_type) {
                                ui.label(value.to_string());
                            } else {
                                ui.label("???");
                            }

                            // Previous value
                            if let Some(prev_bytes) = &result.current_value {
                                if prev_bytes != &result.previous_value {
                                    if let Some(prev_val) =
                                        crate::types::ScanValue::from_bytes(
                                            &result.previous_value,
                                            self.value_type,
                                        )
                                    {
                                        ui.colored_label(
                                            egui::Color32::YELLOW,
                                            prev_val.to_string(),
                                        );
                                    } else {
                                        ui.label("???");
                                    }
                                } else {
                                    ui.label("-");
                                }
                            } else {
                                ui.label("-");
                            }

                            // Actions
                            ui.horizontal(|ui| {
                                if ui.small_button("Edit").clicked() {
                                    self.edit_address = Some(result.address);
                                    if let Some(value) = result.parse_value(self.value_type) {
                                        self.edit_value = value.to_string();
                                    }
                                }

                                if ui.small_button("Refresh").clicked() {
                                    if let Some(scanner) = scanner {
                                        if let Ok(scanner) = scanner.lock() {
                                            if let Ok(_value) =
                                                scanner.read_value(result.address, self.value_type)
                                            {
                                                // Update display (note: this doesn't persist)
                                                ui.ctx().request_repaint();
                                            }
                                        }
                                    }
                                }
                            });

                            ui.end_row();
                        }
                    });
            });

        // Edit value dialog
        if let Some(edit_addr) = self.edit_address {
            egui::Window::new("Edit Value")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Address: 0x{:X}", edit_addr));
                    ui.label(format!("Type: {}", self.value_type));

                    ui.horizontal(|ui| {
                        ui.label("New value:");
                        ui.text_edit_singleline(&mut self.edit_value);
                    });

                    ui.horizontal(|ui| {
                        if ui.button("Write").clicked() {
                            if let Some(scanner) = scanner {
                                if let Ok(scanner) = scanner.lock() {
                                    // Parse and write value
                                    if let Ok(value) = self.parse_value(&self.edit_value) {
                                        if let Err(e) = scanner.write_value(edit_addr, &value) {
                                            tracing::error!("Failed to write value: {}", e);
                                        } else {
                                            self.edit_address = None;
                                        }
                                    }
                                }
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            self.edit_address = None;
                        }
                    });
                });
        }
    }

    fn parse_value(&self, value_str: &str) -> Result<crate::types::ScanValue, String> {
        use crate::types::ScanValue;

        match self.value_type {
            ValueType::I8 => value_str
                .parse::<i8>()
                .map(ScanValue::I8)
                .map_err(|e| e.to_string()),
            ValueType::I16 => value_str
                .parse::<i16>()
                .map(ScanValue::I16)
                .map_err(|e| e.to_string()),
            ValueType::I32 => value_str
                .parse::<i32>()
                .map(ScanValue::I32)
                .map_err(|e| e.to_string()),
            ValueType::I64 => value_str
                .parse::<i64>()
                .map(ScanValue::I64)
                .map_err(|e| e.to_string()),
            ValueType::U8 => value_str
                .parse::<u8>()
                .map(ScanValue::U8)
                .map_err(|e| e.to_string()),
            ValueType::U16 => value_str
                .parse::<u16>()
                .map(ScanValue::U16)
                .map_err(|e| e.to_string()),
            ValueType::U32 => value_str
                .parse::<u32>()
                .map(ScanValue::U32)
                .map_err(|e| e.to_string()),
            ValueType::U64 => value_str
                .parse::<u64>()
                .map(ScanValue::U64)
                .map_err(|e| e.to_string()),
            ValueType::F32 => value_str
                .parse::<f32>()
                .map(ScanValue::F32)
                .map_err(|e| e.to_string()),
            ValueType::F64 => value_str
                .parse::<f64>()
                .map(ScanValue::F64)
                .map_err(|e| e.to_string()),
            ValueType::ByteArray(_) => Err("Byte array editing not yet implemented".to_string()),
        }
    }
}
