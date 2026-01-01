pub struct OmegaScorer;

impl OmegaScorer {
    /// Calculates the Omega Ratio for a given forecast distribution.
    ///
    /// The distribution is approximated as a Triangle defined by (p10, p50, p90).
    /// - threshold: The Minimum Acceptable Return (MAR) (absolute price level).
    ///
    /// Returns:
    /// - Ratio (Area Gain / Area Loss).
    /// - Returns f64::INFINITY if Loss Area is 0.
    pub fn calculate(p10: f64, p50: f64, p90: f64, threshold: f64) -> f64 {
        // Sanity Check
        if p10 >= p90 || p50 < p10 || p50 > p90 {
            // Invalid distribution
            return 0.0;
        }

        // 1. Define Triangle PDF parameters
        let a = p10;
        let c = p50; // Mode
        let b = p90;
        
        // Height of the triangle to ensure Area = 1.0 (PDF property)
        // Area = 0.5 * base * height = 1 => height = 2 / (b - a)
        let h = 2.0 / (b - a);

        // 2. Integration Helper
        // We need Area A (Above Threshold) and Area B (Below Threshold).
        // Instead of full integration, we can calculate the Expected Value of the Gain/Loss directly?
        // Omega = E[max(X - L, 0)] / E[max(L - X, 0)]
        
        let ups = Self::expected_gain(a, c, b, h, threshold);
        let downs = Self::expected_loss(a, c, b, h, threshold);

        if downs == 0.0 {
             if ups > 0.0 { return f64::INFINITY; } else { return 0.0; }
        }

        ups / downs
    }

    /// Expected Gain: Integral of (x - t) * f(x) dx from t to b
    fn expected_gain(a: f64, c: f64, b: f64, h: f64, t: f64) -> f64 {
        if t >= b {
            return 0.0;
        }
        
        // If t < a, we integrate over the whole range [a, b], shifting x by -t
        // But simpler to split cases.
        
        // We integrate (x - t) f(x)
        
        let mut sum = 0.0;

        // Part 1: Increasing Slope (a to c)
        // f(x) = h * (x - a) / (c - a)
        // Integral (x - t) * h * (x - a) / (c - a) dx
        let _upper_1 = if t < c { c } else { t }; // End of relevant range for line 1?
        // Actually, if t > c, this whole part is 0 (since we only care x > t)
        
        if t < c {
            let start = if t > a { t } else { a };
            let end = c;
            
            // Const k = h / (c - a)
            // Int (x - t)(x - a) k dx = k * Int (x^2 - (a+t)x + at) dx
            // = k * [ x^3/3 - (a+t)x^2/2 + atx ] from start to end
            
            let k = h / (c - a);
            sum += k * (Self::poly_integral(end, a, t) - Self::poly_integral(start, a, t));
        }

        // Part 2: Decreasing Slope (c to b)
        // f(x) = h * (b - x) / (b - c)
        // Integral (x - t) * h * (b - x) / (b - c) dx
        // = k' * Int (x - t)(b - x) dx
        // = k' * Int (-x^2 + (b+t)x - bt) dx
        if t < b {
            let start = if t > c { t } else { c };
            let end = b;
            
            let k = h / (b - c);
            sum += k * (Self::poly_integral_down(end, b, t) - Self::poly_integral_down(start, b, t));
        }

        sum
    }

    /// Expected Loss: Integral of (t - x) * f(x) dx from a to t
    fn expected_loss(a: f64, c: f64, b: f64, h: f64, t: f64) -> f64 {
        if t <= a {
            return 0.0;
        }
        
        // Loss = (t - x).
        // Similar logic.
        
        let mut sum = 0.0;

        // Part 1: Increasing Slope (a to c)
        if t > a {
            let start = a;
            let end = if t < c { t } else { c };
            
            let k = h / (c - a);
            // Int (t - x)(x - a) k dx = k * Int (-x^2 + (a+t)x - at) dx
            sum += k * (Self::poly_integral_loss_up(end, a, t) - Self::poly_integral_loss_up(start, a, t));
        }

        // Part 2: Decreasing Slope (c to b)
        if t > c {
            let start = c;
            let end = if t < b { t } else { b };
            
            let k = h / (b - c);
            // Int (t - x)(b - x) k dx = k * Int (x^2 - (b+t)x + bt) dx
            sum += k * (Self::poly_integral_loss_down(end, b, t) - Self::poly_integral_loss_down(start, b, t));
        }

        sum
    }

    // Helpers for integration primitives
    
    // Int (x - t)(x - a) dx = x^3/3 - (a+t)x^2/2 + atx
    fn poly_integral(x: f64, a: f64, t: f64) -> f64 {
        (x.powi(3) / 3.0) - ((a + t) * x.powi(2) / 2.0) + (a * t * x)
    }

    // Int (x - t)(b - x) dx = -x^3/3 + (b+t)x^2/2 - btx
    fn poly_integral_down(x: f64, b: f64, t: f64) -> f64 {
        -(x.powi(3) / 3.0) + ((b + t) * x.powi(2) / 2.0) - (b * t * x)
    }

    // Int (t - x)(x - a) dx = -x^3/3 + (a+t)x^2/2 - atx  (Negation of poly_integral)
    fn poly_integral_loss_up(x: f64, a: f64, t: f64) -> f64 {
        -(x.powi(3) / 3.0) + ((a + t) * x.powi(2) / 2.0) - (a * t * x)
    }

    // Int (t - x)(b - x) dx = x^3/3 - (b+t)x^2/2 + btx (Negation of poly_integral_down)
    fn poly_integral_loss_down(x: f64, b: f64, t: f64) -> f64 {
        (x.powi(3) / 3.0) - ((b + t) * x.powi(2) / 2.0) + (b * t * x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_omega_basic() {
        // Symmetric Triangle around 100. Range 90-110. P50=100.
        // If threshold = 100. Upside = Downside. Omega should be 1.0.
        let omega = OmegaScorer::calculate(90.0, 100.0, 110.0, 100.0);
        assert!((omega - 1.0).abs() < 0.001, "Omega should be 1.0 for symmetric split, got {}", omega);
    }

    #[test]
    fn test_omega_bullish() {
        // Skewed Up. Range 95-120. P50=100. Threshold = 100.
        // More mass above 100. Omega > 1.0.
        let omega = OmegaScorer::calculate(95.0, 100.0, 120.0, 100.0);
        assert!(omega > 1.0, "Omega should be > 1.0 for bullish skew");
    }

    #[test]
    fn test_omega_bearish() {
        // Skewed Down. Range 80-105. P50=100. Threshold = 100.
        // More mass below 100. Omega < 1.0.
        let omega = OmegaScorer::calculate(80.0, 100.0, 105.0, 100.0);
        assert!(omega < 1.0, "Omega should be < 1.0 for bearish skew");
    }

    #[test]
    fn test_veto_level() {
        // Case where risk is high.
        // P10=90, P50=101, P90=105. Threshold=100.
        // Upside: 100 -> 105 (small). Downside: 90 -> 100 (large).
        let omega = OmegaScorer::calculate(90.0, 101.0, 105.0, 100.0);
        println!("Bearish Omega: {}", omega);
        assert!(omega < 1.0);
    }
}
