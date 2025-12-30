use crate::types::{ScanType, ScanValue};

/// Engine for filtering scan results
pub struct FilterEngine;

impl FilterEngine {
    /// Checks if a value matches the filter criteria
    pub fn matches(
        current: &ScanValue,
        previous: Option<&ScanValue>,
        target: Option<&ScanValue>,
        scan_type: ScanType,
    ) -> bool {
        match scan_type {
            ScanType::Exact | ScanType::GreaterThan | ScanType::LessThan => {
                if let Some(target_val) = target {
                    current.compare(target_val, scan_type)
                } else {
                    false
                }
            }
            ScanType::Between(_, _) => {
                if let Some(target_val) = target {
                    current.compare(target_val, scan_type)
                } else {
                    false
                }
            }
            ScanType::Increased | ScanType::Decreased | ScanType::Changed | ScanType::Unchanged => {
                if let Some(prev_val) = previous {
                    match scan_type {
                        ScanType::Increased => current.as_f64() > prev_val.as_f64(),
                        ScanType::Decreased => current.as_f64() < prev_val.as_f64(),
                        ScanType::Changed => current != prev_val,
                        ScanType::Unchanged => current == prev_val,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            ScanType::Unknown => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let value = ScanValue::I32(100);
        let target = ScanValue::I32(100);

        assert!(FilterEngine::matches(
            &value,
            None,
            Some(&target),
            ScanType::Exact
        ));
    }

    #[test]
    fn test_increased() {
        let current = ScanValue::I32(150);
        let previous = ScanValue::I32(100);

        assert!(FilterEngine::matches(
            &current,
            Some(&previous),
            None,
            ScanType::Increased
        ));
    }

    #[test]
    fn test_decreased() {
        let current = ScanValue::I32(50);
        let previous = ScanValue::I32(100);

        assert!(FilterEngine::matches(
            &current,
            Some(&previous),
            None,
            ScanType::Decreased
        ));
    }
}
