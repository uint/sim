use rand::distributions::Distribution;
use serde::{Deserialize, Serialize};
// Continuous distributions
use rand_distr::{Beta, Exp, Gamma, LogNormal, Normal, Triangular, Uniform, Weibull};
// Discrete distributions
use rand_distr::{Bernoulli, Geometric, Poisson, WeightedIndex};

use super::uniform_rng::UniformRNG;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RandomVariable {
    // Continuous distributions
    Beta { alpha: f64, beta: f64 },
    Exp { lambda: f64 },
    Gamma { shape: f64, scale: f64 },
    LogNormal { mu: f64, sigma: f64 },
    Normal { mean: f64, std_dev: f64 },
    Triangular { min: f64, max: f64, mode: f64 },
    Uniform { min: f64, max: f64 },
    Weibull { shape: f64, scale: f64 },
    // Discrete distributions
    Bernoulli { p: f64 },
    Geometric { p: f64 },
    Poisson { lambda: f64 },
    WeightedIndex { weights: Vec<u64> },
}

impl RandomVariable {
    pub fn random_variate(&mut self, uniform_rng: &mut UniformRNG) -> f64 {
        match self {
            RandomVariable::Beta { alpha, beta } => {
                Beta::new(*alpha, *beta).unwrap().sample(uniform_rng.rng())
            }
            RandomVariable::Exp { lambda } => Exp::new(*lambda).unwrap().sample(uniform_rng.rng()),
            RandomVariable::Gamma { shape, scale } => Gamma::new(*shape, *scale)
                .unwrap()
                .sample(uniform_rng.rng()),
            RandomVariable::LogNormal { mu, sigma } => LogNormal::new(*mu, *sigma)
                .unwrap()
                .sample(uniform_rng.rng()),
            RandomVariable::Normal { mean, std_dev } => Normal::new(*mean, *std_dev)
                .unwrap()
                .sample(uniform_rng.rng()),
            RandomVariable::Triangular { min, max, mode } => Triangular::new(*min, *max, *mode)
                .unwrap()
                .sample(uniform_rng.rng()),
            RandomVariable::Uniform { min, max } => {
                Uniform::new(*min, *max).sample(uniform_rng.rng())
            }
            RandomVariable::Weibull { shape, scale } => Weibull::new(*shape, *scale)
                .unwrap()
                .sample(uniform_rng.rng()),
            RandomVariable::Bernoulli { p } => {
                Bernoulli::new(*p).unwrap().sample(uniform_rng.rng()) as u8 as f64
            }
            RandomVariable::Geometric { p } => {
                Geometric::new(*p).unwrap().sample(uniform_rng.rng()) as f64
            }
            RandomVariable::Poisson { lambda } => {
                Poisson::new(*lambda).unwrap().sample(uniform_rng.rng())
            }
            RandomVariable::WeightedIndex { weights } => WeightedIndex::new(weights.clone())
                .unwrap()
                .sample(uniform_rng.rng())
                as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beta_samples_match_expectation() {
        let mut variable = RandomVariable::Beta {
            alpha: 7.0,
            beta: 11.0,
        };
        let mut uniform_rng = UniformRNG::default();
        let mean = (0..10000)
            .map(|_| variable.random_variate(&mut uniform_rng))
            .sum::<f64>()
            / 10000.0;
        let expected = 7.0 / (7.0 + 11.0);
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn exponential_samples_match_expectation() {
        let mut variable = RandomVariable::Exp { lambda: 7.0 };
        let mut uniform_rng = UniformRNG::default();
        let mean = (0..10000)
            .map(|_| variable.random_variate(&mut uniform_rng))
            .sum::<f64>()
            / 10000.0;
        let expected = 1.0 / 7.0;
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn gamma_samples_match_expectation() {
        let mut variable = RandomVariable::Gamma {
            shape: 7.0,
            scale: 11.0,
        };
        let mut uniform_rng = UniformRNG::default();
        let mean = (0..10000)
            .map(|_| variable.random_variate(&mut uniform_rng))
            .sum::<f64>()
            / 10000.0;
        let expected = 77.0;
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn lognormal_samples_match_expectation() {
        let mut variable = RandomVariable::LogNormal {
            mu: 11.0,
            sigma: 1.0,
        };
        let mut uniform_rng = UniformRNG::default();
        let mean = (0..10000)
            .map(|_| variable.random_variate(&mut uniform_rng))
            .sum::<f64>()
            / 10000.0;
        let expected = (11.0f64 + 1.0f64.powi(2) / 2.0f64).exp();
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn normal_samples_chi_square() {
        let mut variable = RandomVariable::Normal {
            mean: 11.0,
            std_dev: 3.0,
        };
        // 8 classes (a.k.a. bins)
        // On each side: within 1 sigma, 1 sigma to 2 sigma, 2 sigma to 3 sigma, 3+ sigma
        let mut class_counts: [f64; 8] = [0.0; 8];
        let mut uniform_rng = UniformRNG::default();
        (0..10000).for_each(|_| {
            let variate = variable.random_variate(&mut uniform_rng);
            if variate < 2.0 {
                class_counts[0] += 1.0;
            } else if variate < 5.0 {
                class_counts[1] += 1.0;
            } else if variate < 8.0 {
                class_counts[2] += 1.0;
            } else if variate < 11.0 {
                class_counts[3] += 1.0;
            } else if variate < 14.0 {
                class_counts[4] += 1.0;
            } else if variate < 17.0 {
                class_counts[5] += 1.0;
            } else if variate < 20.0 {
                class_counts[6] += 1.0;
            } else {
                class_counts[7] += 1.0;
            }
        });
        let expected_counts: [f64; 8] = [20.0, 210.0, 1360.0, 3410.0, 3410.0, 1360.0, 210.0, 20.0];
        let chi_square = class_counts.iter().zip(expected_counts.iter()).fold(
            0.0,
            |acc, (class_count, expected_count)| {
                acc + (*class_count - expected_count).powi(2) / expected_count
            },
        );
        // At a significance level of 0.01, and with n-1=7 degrees of freedom, the chi square critical
        // value for this scenario is 18.475
        let chi_square_critical = 18.475;
        assert![chi_square < chi_square_critical];
    }

    #[test]
    fn trianglular_samples_chi_square() {
        let mut variable = RandomVariable::Triangular {
            min: 5.0,
            max: 25.0,
            mode: 15.0,
        };
        // 4 classes/bins - each of width 5
        let mut class_counts: [f64; 4] = [0.0; 4];
        let mut uniform_rng = UniformRNG::default();
        (0..1000).for_each(|_| {
            let variate = variable.random_variate(&mut uniform_rng);
            class_counts[((variate - 5.0) / 5.0) as usize] += 1.0;
        });
        let expected_counts: [f64; 4] = [125.0, 375.0, 375.0, 125.0];
        let chi_square = class_counts.iter().zip(expected_counts.iter()).fold(
            0.0,
            |acc, (class_count, expected_count)| {
                acc + (*class_count - expected_count).powi(2) / expected_count
            },
        );
        // At a significance level of 0.01, and with n-1=3 degrees of freedom, the chi square critical
        // value for this scenario is 134.642
        let chi_square_critical = 11.345;
        assert![chi_square < chi_square_critical];
    }

    #[test]
    fn uniform_samples_chi_square() {
        let mut variable = RandomVariable::Uniform {
            min: 7.0,
            max: 11.0,
        };
        let mut class_counts: [f64; 40] = [0.0; 40];
        let mut uniform_rng = UniformRNG::default();
        (0..10000).for_each(|_| {
            let rn = variable.random_variate(&mut uniform_rng);
            let class_index = (rn - 7.0) * 10.0;
            class_counts[class_index as usize] += 1.0;
        });
        let expected_counts: [f64; 40] = [250.0; 40];
        let chi_square = class_counts.iter().zip(expected_counts.iter()).fold(
            0.0,
            |acc, (class_count, expected_count)| {
                acc + (*class_count - expected_count).powi(2) / expected_count
            },
        );
        // At a significance level of 0.01, and with n-1=39 degrees of freedom, the chi square critical
        // value for this scenario is 62.428
        let chi_square_critical = 62.428;
        assert![chi_square < chi_square_critical];
    }

    #[test]
    fn weibull_samples_match_expectation() {
        let mut variable = RandomVariable::Weibull {
            shape: 7.0,
            scale: 0.5,
        };
        let mut uniform_rng = UniformRNG::default();
        let mean = (0..10000)
            .map(|_| variable.random_variate(&mut uniform_rng))
            .sum::<f64>()
            / 10000.0;
        let expected = 14.0;
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn bernoulli_samples_chi_square() {
        let mut variable = RandomVariable::Bernoulli { p: 0.3 };
        let mut class_counts: [f64; 2] = [0.0; 2];
        let mut uniform_rng = UniformRNG::default();
        (0..10000).for_each(|_| {
            let rn = variable.random_variate(&mut uniform_rng);
            class_counts[rn as usize] += 1.0;
        });
        let expected_counts: [f64; 2] = [7000.0, 3000.0];
        let chi_square = class_counts.iter().zip(expected_counts.iter()).fold(
            0.0,
            |acc, (class_count, expected_count)| {
                acc + (*class_count - expected_count).powi(2) / expected_count
            },
        );
        // At a significance level of 0.01, and with n-1=1 degrees of freedom, the chi square critical
        // value for this scenario is 6.635
        let chi_square_critical = 6.635;
        assert![chi_square < chi_square_critical];
    }

    #[test]
    fn geometric_samples_match_expectation() {
        let mut variable = RandomVariable::Geometric { p: 0.2 };
        let mut uniform_rng = UniformRNG::default();
        let mean = (0..10000)
            .map(|_| variable.random_variate(&mut uniform_rng))
            .sum::<f64>()
            / 10000.0;
        let expected = (1.0 - 0.2) / 0.2;
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn poisson_samples_match_expectation() {
        let mut variable = RandomVariable::Poisson { lambda: 7.0 };
        let mut uniform_rng = UniformRNG::default();
        let mean = (0..10000)
            .map(|_| variable.random_variate(&mut uniform_rng))
            .sum::<f64>()
            / 10000.0;
        let expected = 7.0;
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn weighted_index_samples_chi_square() {
        let mut variable = RandomVariable::WeightedIndex {
            weights: vec![1, 2, 3, 4],
        };
        let mut class_counts: [f64; 4] = [0.0; 4];
        let mut uniform_rng = UniformRNG::default();
        (0..10000).for_each(|_| {
            let rn = variable.random_variate(&mut uniform_rng);
            class_counts[rn as usize] += 1.0;
        });
        let expected_counts: [f64; 4] = [1000.0, 2000.0, 3000.0, 4000.0];
        let chi_square = class_counts.iter().zip(expected_counts.iter()).fold(
            0.0,
            |acc, (class_count, expected_count)| {
                acc + (*class_count - expected_count).powi(2) / expected_count
            },
        );
        // At a significance level of 0.01, and with n-1=3 degrees of freedom, the chi square critical
        // value for this scenario is 11.345
        let chi_square_critical = 11.345;
        assert![chi_square < chi_square_critical];
    }
}
