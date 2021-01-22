pub fn sample_mean(points: &[f64]) -> f64 {
    points.iter().sum::<f64>() / (points.len() as f64)
}

pub fn sample_variance(points: &[f64], mean: &f64) -> f64 {
    points
        .iter()
        .fold(0.0, |acc, point| acc + (point - mean).powi(2))
        / (points.len() as f64)
}

pub fn equivalent_f64(a: f64, b: f64) -> bool {
    a - b == 0.0
}

pub fn evaluate_polynomial(coefficients: &[f64], x: f64) -> f64 {
    // Horner's method for polynomial evlauation
    let highest_order_polynomial_coeff = coefficients.first().unwrap();
    coefficients[0..coefficients.len() - 1]
        .iter()
        .fold(*highest_order_polynomial_coeff, |acc, coefficient| {
            coefficient + x * acc
        })
}