use crate::normalized_value::is_normalized_value;
use reaper_medium::Bpm;

const BPM_SPAN: f64 = Bpm::MAX.get() - Bpm::MIN.get();

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tempo(Bpm);

impl Tempo {
    pub fn from_bpm(bpm: Bpm) -> Tempo {
        Tempo(bpm)
    }

    pub fn from_normalized_value(normalized_value: f64) -> Tempo {
        assert!(is_normalized_value(normalized_value));
        Tempo(Bpm::new(Bpm::MIN.get() + normalized_value * BPM_SPAN))
    }

    pub fn normalized_value(self) -> f64 {
        (self.0.get() - Bpm::MIN.get()) / BPM_SPAN
    }

    pub fn bpm(self) -> Bpm {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bpm() {
        // Given
        let tempo = Tempo::from_bpm(Bpm::new(120.0));
        // Then
        assert_eq!(tempo.bpm(), Bpm::new(120.0));
        let normalized_value = tempo.normalized_value();
        assert!(0.1240 < normalized_value && normalized_value < 0.1241);
    }

    #[test]
    fn from_normalized_value() {
        // Given
        let tempo = Tempo::from_normalized_value(0.5);
        // Then
        assert_eq!(tempo.bpm(), Bpm::new(480.5));
    }
}
