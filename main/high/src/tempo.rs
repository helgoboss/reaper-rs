use crate::normalized_value::is_normalized_value;
use reaper_medium::Bpm;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tempo(Bpm);

impl Tempo {
    pub fn from_bpm(bpm: Bpm) -> Tempo {
        Tempo(bpm)
    }

    pub fn from_normalized_value(normalized_value: f64) -> Tempo {
        assert!(is_normalized_value(normalized_value));
        Tempo(Bpm::new_panic(
            Bpm::MIN.get() + normalized_value * bpm_span(),
        ))
    }

    pub fn normalized_value(self) -> f64 {
        (self.0.get() - Bpm::MIN.get()) / bpm_span()
    }

    pub fn bpm(self) -> Bpm {
        self.0
    }
}

fn bpm_span() -> f64 {
    Bpm::MAX.get() - Bpm::MIN.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bpm() {
        // Given
        let tempo = Tempo::from_bpm(Bpm::new_panic(120.0));
        // Then
        assert_eq!(tempo.bpm(), Bpm::new_panic(120.0));
        let normalized_value = tempo.normalized_value();
        assert!(0.1240 < normalized_value && normalized_value < 0.1241);
    }

    #[test]
    fn from_normalized_value() {
        // Given
        let tempo = Tempo::from_normalized_value(0.5);
        // Then
        assert_eq!(tempo.bpm(), Bpm::new_panic(480.5));
    }
}
