use crate::types::{ScanType, ValueType};
use eframe::egui;

/// UI component for scan configuration
pub struct ScanView {
    pub value_input: String,
    pub selected_value_type: ValueType,
    pub selected_scan_type: ScanType,
    pub alignment: usize,
}

impl Default for ScanView {
    fn default() -> Self {
        Self {
            value_input: String::new(),
            selected_value_type: ValueType::I32,
            selected_scan_type: ScanType::Exact,
            alignment: 4,
        }
    }
}

impl ScanView {
    pub fn reset(&mut self) {
        self.value_input.clear();
        self.selected_scan_type = ScanType::Exact;
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Value input
        ui.horizontal(|ui| {
            ui.label("Value:");
            ui.text_edit_singleline(&mut self.value_input);
        });

        // Value type selector
        ui.horizontal(|ui| {
            ui.label("Type:");
            egui::ComboBox::new("value_type", "")
                .selected_text(self.selected_value_type.display_name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_value_type, ValueType::I8, "Int8");
                    ui.selectable_value(&mut self.selected_value_type, ValueType::I16, "Int16");
                    ui.selectable_value(&mut self.selected_value_type, ValueType::I32, "Int32");
                    ui.selectable_value(&mut self.selected_value_type, ValueType::I64, "Int64");
                    ui.selectable_value(&mut self.selected_value_type, ValueType::U8, "UInt8");
                    ui.selectable_value(&mut self.selected_value_type, ValueType::U16, "UInt16");
                    ui.selectable_value(&mut self.selected_value_type, ValueType::U32, "UInt32");
                    ui.selectable_value(&mut self.selected_value_type, ValueType::U64, "UInt64");
                    ui.selectable_value(&mut self.selected_value_type, ValueType::F32, "Float");
                    ui.selectable_value(&mut self.selected_value_type, ValueType::F64, "Double");
                });
        });

        // Update alignment when type changes
        self.alignment = self.selected_value_type.alignment();

        // Scan type selector
        ui.horizontal(|ui| {
            ui.label("Scan:");
            egui::ComboBox::new("scan_type", "")
                .selected_text(self.selected_scan_type.display_name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.selected_scan_type,
                        ScanType::Exact,
                        "Exact Value",
                    );
                    ui.selectable_value(
                        &mut self.selected_scan_type,
                        ScanType::GreaterThan,
                        "Greater Than",
                    );
                    ui.selectable_value(
                        &mut self.selected_scan_type,
                        ScanType::LessThan,
                        "Less Than",
                    );
                    ui.selectable_value(
                        &mut self.selected_scan_type,
                        ScanType::Unknown,
                        "Unknown Initial Value",
                    );
                    ui.separator();
                    ui.selectable_value(
                        &mut self.selected_scan_type,
                        ScanType::Increased,
                        "Increased",
                    );
                    ui.selectable_value(
                        &mut self.selected_scan_type,
                        ScanType::Decreased,
                        "Decreased",
                    );
                    ui.selectable_value(
                        &mut self.selected_scan_type,
                        ScanType::Changed,
                        "Changed",
                    );
                    ui.selectable_value(
                        &mut self.selected_scan_type,
                        ScanType::Unchanged,
                        "Unchanged",
                    );
                });
        });

        // Alignment option (advanced)
        ui.collapsing("Advanced", |ui| {
            ui.horizontal(|ui| {
                ui.label("Alignment:");
                ui.add(egui::DragValue::new(&mut self.alignment).range(1..=16));
            });
        });
    }
}
