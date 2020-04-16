use crate::normalized_value::is_normalized_value;

type Bpm = f64;

pub const MAX_BPM: f64 = 960.0;
pub const MIN_BPM: f64 = 1.0;
const BPM_SPAN: f64 = MAX_BPM - MIN_BPM;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tempo {
    bpm: Bpm,
}

pub fn is_bpm_value(value: f64) -> bool {
    MIN_BPM <= value && value <= MAX_BPM
}

impl Tempo {
    pub fn from_bpm(bpm: Bpm) -> Tempo {
        assert!(is_bpm_value(bpm));
        Tempo { bpm }
    }

    pub fn from_normalized_value(normalized_value: f64) -> Tempo {
        assert!(is_normalized_value(normalized_value));
        Tempo {
            bpm: MIN_BPM + normalized_value * BPM_SPAN,
        }
    }

    pub fn get_normalized_value(&self) -> f64 {
        (self.bpm - MIN_BPM) / BPM_SPAN
    }

    pub fn get_bpm(&self) -> Bpm {
        self.bpm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bpm() {
        // Given
        let tempo = Tempo::from_bpm(120.0);
        // Then
        assert_eq!(tempo.get_bpm(), 120.0);
        let normalized_value = tempo.get_normalized_value();
        assert!(0.1240 < normalized_value && normalized_value < 0.1241);
    }

    #[test]
    fn from_normalized_value() {
        // Given
        let tempo = Tempo::from_normalized_value(0.5);
        // Then
        assert_eq!(tempo.get_bpm(), 480.5);
    }
}
