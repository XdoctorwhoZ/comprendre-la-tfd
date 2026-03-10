use std::f64::consts::PI;

/// Configuration for one sinusoidal component: A * sin(ω*x + φ)
///
/// Maintains synchronisation between the angular frequency (coeff_a = ω)
/// and the linear frequency (frequency = f = ω / 2π).
#[derive(Clone, Debug, PartialEq)]
pub struct SinusConfig {
    /// Angular frequency ω = 2π·f
    pub coeff_a: f64,
    /// Phase offset φ
    pub coeff_b: f64,
    /// Linear frequency f (Hz)
    pub frequency: f64,
}

impl Default for SinusConfig {
    fn default() -> Self {
        let frequency = 1.0_f64;
        Self {
            coeff_a: 2.0 * PI * frequency,
            coeff_b: 0.0,
            frequency,
        }
    }
}

impl SinusConfig {
    /// Evaluate sin(ω·x + φ) at position x
    pub fn sin_val(&self, x: f64) -> f64 {
        (self.coeff_a * x + self.coeff_b).sin()
    }

    /// Period T = 1 / f
    pub fn periode(&self) -> f64 {
        if self.frequency > 1e-10 {
            1.0 / self.frequency
        } else {
            1.0
        }
    }

    /// Create a new config from an angular frequency value, keeping coeff_b
    pub fn with_coeff_a(coeff_b: f64, coeff_a: f64) -> Self {
        let frequency = if coeff_a > 1e-10 {
            coeff_a / (2.0 * PI)
        } else {
            0.0
        };
        Self { coeff_a, coeff_b, frequency }
    }

    /// Create a new config from a frequency value, keeping coeff_b
    pub fn with_frequency(coeff_b: f64, frequency: f64) -> Self {
        let coeff_a = 2.0 * PI * frequency;
        Self { coeff_a, coeff_b, frequency }
    }
}

/// Compute the sum of active sine waves at position x
pub fn signal_at(x: f64, configs: &[SinusConfig]) -> f64 {
    configs.iter().map(|c| c.sin_val(x)).sum()
}

/// Find the largest period among configs (returns 1.0 if configs is empty)
pub fn max_periode(configs: &[SinusConfig]) -> f64 {
    configs
        .iter()
        .map(|c| c.periode())
        .fold(0.0_f64, f64::max)
        .max(1e-10)
}

/// Sample the signal at N equally-spaced time steps with sampling frequency fe
pub fn sample_signal(configs: &[SinusConfig], n: usize, fe: f64) -> Vec<f64> {
    if fe < 1e-10 || n == 0 {
        return vec![0.0; n.max(1)];
    }
    let t_step = 1.0 / fe;
    (0..n)
        .map(|i| signal_at(i as f64 * t_step, configs))
        .collect()
}

/// Compute the DFT magnitude spectrum using the naive O(N²) formula.
///
/// X[k] = Σ_{n=0}^{N-1} x[n] · e^{−j 2π k n / N}
/// |X[k]| = √(Re² + Im²)
pub fn compute_dft(samples: &[f64]) -> Vec<f64> {
    let n = samples.len();
    if n == 0 {
        return vec![];
    }
    (0..n)
        .map(|k| {
            let mut real = 0.0_f64;
            let mut imag = 0.0_f64;
            for (ni, &sample) in samples.iter().enumerate() {
                let angle = -2.0 * PI * k as f64 * ni as f64 / n as f64;
                real += sample * angle.cos();
                imag += sample * angle.sin();
            }
            (real * real + imag * imag).sqrt()
        })
        .collect()
}

/// Generate x and y vectors for the continuous signal plot spanning 3 periods.
pub fn generate_plot_signal(
    configs: &[SinusConfig],
    nb_points: usize,
) -> (Vec<f64>, Vec<f64>) {
    let max_t = max_periode(configs);
    let step = (3.0 * max_t) / nb_points as f64;
    let x: Vec<f64> = (0..nb_points).map(|i| i as f64 * step).collect();
    let y: Vec<f64> = x.iter().map(|&xi| signal_at(xi, configs)).collect();
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sinus_config_default() {
        let c = SinusConfig::default();
        assert!((c.frequency - 1.0).abs() < 1e-10);
        assert!((c.coeff_a - 2.0 * PI).abs() < 1e-10);
        assert_eq!(c.coeff_b, 0.0);
    }

    #[test]
    fn test_sinus_config_periode() {
        let c = SinusConfig::default();
        // period of 1 Hz sine = 1 second
        assert!((c.periode() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sin_val_at_zero() {
        let c = SinusConfig::default(); // sin(ω·0 + 0) = 0
        assert!((c.sin_val(0.0)).abs() < 1e-10);
    }

    #[test]
    fn test_sin_val_quarter_period() {
        let c = SinusConfig::default(); // 1 Hz => T=1s, quarter period = 0.25s
        let quarter = c.periode() / 4.0; // sin(2π·0.25) = sin(π/2) = 1
        assert!((c.sin_val(quarter) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_signal_at_sum() {
        let configs = vec![
            SinusConfig::default(),
            SinusConfig::default(),
        ];
        // Two identical 1 Hz sines => sum at 0 = 0
        assert!((signal_at(0.0, &configs)).abs() < 1e-10);
    }

    #[test]
    fn test_sample_signal_length() {
        let configs = vec![SinusConfig::default()];
        let samples = sample_signal(&configs, 64, 10.0);
        assert_eq!(samples.len(), 64);
    }

    #[test]
    fn test_sample_signal_zero_fe() {
        let configs = vec![SinusConfig::default()];
        let samples = sample_signal(&configs, 10, 0.0);
        assert!(samples.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_compute_dft_dc() {
        // DFT of a constant signal [1, 1, 1, 1] should have X[0] = N = 4
        let samples = vec![1.0_f64; 4];
        let dft = compute_dft(&samples);
        assert_eq!(dft.len(), 4);
        assert!((dft[0] - 4.0).abs() < 1e-6);
        // All other bins should be ~0
        for &v in &dft[1..] {
            assert!(v < 1e-6);
        }
    }

    #[test]
    fn test_compute_dft_empty() {
        let dft = compute_dft(&[]);
        assert!(dft.is_empty());
    }

    #[test]
    fn test_with_frequency_sync() {
        let c = SinusConfig::with_frequency(0.0, 2.0);
        assert!((c.frequency - 2.0).abs() < 1e-10);
        assert!((c.coeff_a - 4.0 * PI).abs() < 1e-10);
    }

    #[test]
    fn test_with_coeff_a_sync() {
        let c = SinusConfig::with_coeff_a(0.0, 4.0 * PI);
        assert!((c.coeff_a - 4.0 * PI).abs() < 1e-10);
        assert!((c.frequency - 2.0).abs() < 1e-10);
    }
}
