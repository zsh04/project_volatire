use ndarray::{Array1, Array2};
use ndarray_rand::rand_distr::Uniform;
use ndarray_rand::RandomExt;
use rand::thread_rng;

pub struct EchoStateNetwork {
    // Hyperparameters
    pub reservoir_size: usize,
    pub forgetting_factor: f64, // Lambda (e.g. 0.99)
    pub spectral_radius: f64,   // e.g. 0.9

    // Weights
    w_in: Array2<f64>,  // [size, 1] - Fixed
    w_res: Array2<f64>, // [size, size] - Fixed
    w_out: Array1<f64>, // [size] - Learned

    // State
    x: Array1<f64>, // [size]
    
    // RLS Covariance Matrix
    p: Array2<f64>, // [size, size]
}

impl EchoStateNetwork {
    pub fn new(size: usize) -> Self {
        let _rng = thread_rng();
        
        // 1. Initialize Input Weights (Uniform -0.5, 0.5)
        let w_in = Array2::random((size, 1), Uniform::new(-0.5, 0.5));

        // 2. Initialize Reservoir Weights (Sparse, Spectral Radius scaled)
        let sparsity = 0.1; // 10% connectivity
        
        // Populate manual sparsity
        let dist = Uniform::new(-1.0, 1.0);
        
        // We can't easily iterate random indices purely with ndarray-rand for sparsity 
        // without a mask. Let's just loop for simplicity of implementation vs external crates.
        // Actually, just fill all then mask? No, N=100 is small.
        // Let's make dense then mask?
        let mut dense = Array2::random((size, size), dist);
        // Naive eigenvalue scaling: Divide by max singular value or just trace? 
        // A common heuristic is dividing by largest absolute row sum.
        // Or just fixed scaling factor that works empirically. 
        // Factor = 0.9.
        let scaling = 0.9 / (size as f64).sqrt(); // Heuristic for spectral radius ~ 1
        dense *= scaling;

        // Apply sparsity (zero out 90%)
        // This is a rough way to do it.
        for val in dense.iter_mut() {
            if rand::random::<f64>() > sparsity {
                *val = 0.0;
            }
        }
        let w_res = dense;

        // 3. RLS Initialization
        // P = 1000 * I (High uncertainty)
        let p = Array2::eye(size) * 1000.0;
        
        // W_out = 0
        let w_out = Array1::zeros(size);

        // x = 0
        let x = Array1::zeros(size);

        Self {
            reservoir_size: size,
            forgetting_factor: 0.99,
            spectral_radius: 0.9,
            w_in,
            w_res,
            w_out,
            x,
            p,
        }
    }

    /// Forward Pass: Update Reservoir State & Predict
    /// Returns predicted next value
    pub fn forward(&mut self, input: f64) -> f64 {
        let alpha = 0.3; // Leaking rate
        
        // u_t is scalar input, but we need vector for matmul
        // w_in * u
        let input_term = &self.w_in * input; // [size, 1]
        let input_vec = input_term.column(0); // [size]

        // res_term = W_res * x_{t-1}
        let res_term = self.w_res.dot(&self.x);

        // activation = tanh(in + res)
        let activation = (&input_vec.to_owned() + &res_term).mapv(f64::tanh);

        // x_t = (1-a)x_{t-1} + a*activation
        self.x = (&self.x * (1.0 - alpha)) + (activation * alpha);

        // Predict y = W_out^T * x_t
        // Dot product
        self.w_out.dot(&self.x)
    }

    /// Train (RLS): Update weights based on error
    /// target: The ACTUAL value that just happened (that we tried to predict prev tick)
    /// Note: `forward` gives prediction for NEXT tick.
    /// So workflow is: 
    /// 1. `pred = forward(current_val)`
    /// 2. Wait for next tick.
    /// 3. `train(next_val)` (using state from step 1... wait, RLS usually uses state that GENERATED prediction)
    ///
    /// For RLS to work correctly, we update based on the error of the *previous* prediction.
    /// Does `train` need the old state? 
    /// If we call `forward` then `train` immediately, we are training on the current state to map to current target? NO.
    /// We usually map x_t -> y_{t+1}.
    /// So we need distinct steps.
    /// 
    /// SIMPLIFIED FLOW:
    /// `forward(input_t)` -> Updates state to `x_t`. Predicts `y_{t+1}`.
    /// ... tick happens ... input_{t+1} arrives.
    /// We now know `y_{t+1}` (it's input_{t+1}).
    /// We should `train(input_{t+1})` using `x_t`. 
    /// 
    /// ISSUE: `x` is already overwritten by next `forward` if we are not careful.
    /// 
    /// Proposed usage in main:
    /// 1. `simons.train(current_tick_val)` -> Updates weights using `prev_x` and `current_tick_val`.
    /// 2. `pred = simons.forward(current_tick_val)` -> Updates `x` to `next_x`, predicts `next_tick_val`.
    /// 
    /// So `train` needs to happen BEFORE `forward` updates the state, but using the state from BEFORE forward?
    /// Actually:
    /// t=0. x=0. forward(v0) -> x1. pred_v1.
    /// t=1. v1 arrives. 
    /// train(v1) should use x1? Yes. x1 produced pred_v1.
    /// forward(v1) -> x2. pred_v2.
    /// 
    /// So `train` should use CURRENT `self.x` (which is x_{t-1} from perspective of new tick? No, `x` persists).
    /// If `forward` updates x, then `train` must be called BEFORE `forward` if `x` assumes it holds the state that made the prediction.
    /// But wait. At t=1 arrival:
    /// We hold x1 (computed at t=0).
    /// We verify pred_v1 vs v1.
    /// We train W_out to map x1 -> v1.
    /// Then we update x1 -> x2 using v1.
    /// 
    /// CORRECT ORDER:
    /// 1. `train(current_input)` (Uses `self.x`).
    /// 2. `forward(current_input)` (Updates `self.x`).
    pub fn train(&mut self, target: f64) {
        // RLS Algorithm
        // e = target - y_hat
        // But y_hat = w_out . x
        let prediction = self.w_out.dot(&self.x);
        let error = target - prediction;

        // k (Gain) = (P * x) / (lambda + x^T * P * x)
        // Numerator: vector [size]
        let p_dot_x = self.p.dot(&self.x); 
        
        // Denom: scalar
        let x_dot_p_dot_x = self.x.dot(&p_dot_x);
        let denom = self.forgetting_factor + x_dot_p_dot_x;
        
        let k = &p_dot_x / denom; // Vector

        // Update Weights: w = w + k * e
        // (Is it minus or plus? y = w.x. e = d - y. rule w += e*x. yes plus.)
        self.w_out = &self.w_out + &(&k * error);

        // Update P: P = (P - k * x^T * P) / lambda
        // k * (x^T * P) -> Outer product?
        // x^T * P is row vector (same as p_dot_x transpose if symmetric).
        // k is col vector. k * row -> Matrix.
        
        // Let's compute term: k [N] * p_dot_x [N] (outer product)
        // ndarray outer product:
        // shape (N,1) * (1,N) -> (N,N)
        // Let's being explicit with shapes if needed, or loop.
        // ndarray doesn't have simple `outer` for 1D arrays, need to reshape.
        let k_2d = k.clone().into_shape((self.reservoir_size, 1)).unwrap();
        let px_2d = p_dot_x.into_shape((1, self.reservoir_size)).unwrap();
        let correction = k_2d.dot(&px_2d);
        
        self.p = (&self.p - &correction) / self.forgetting_factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convergence() {
        let mut esn = EchoStateNetwork::new(50);
        let mut total_error = 0.0;
        
        // Pattern: 1.0, -1.0, 1.0, -1.0
        // We want to predict Next from Current.
        // If current is 1.0, next is -1.0.
        // If current is -1.0, next is 1.0.
        
        let prev_input = 1.0;
        
        // Burn-in (reservoir settle)
        esn.forward(prev_input);

        // Train loop
        for i in 0..200 {
            let input = if i % 2 == 0 { -1.0 } else { 1.0 };
            
            // 1. Train on what just happened (input is the target for the previous prediction)
            esn.train(input);
            
            // 2. Predict next
            let pred = esn.forward(input);
            
            // Check error (Next actual will be opposing sign)
            let next_actual = if input == 1.0 { -1.0 } else { 1.0 };
            let err = (next_actual - pred).abs();
            
            if i > 150 {
                total_error += err;
                println!("Step {}: In={} Act={} Pred={} Err={}", i, input, next_actual, pred, err);
            }
        }
        
        let avg_error = total_error / 50.0;
        println!("Avg Error last 50 steps: {}", avg_error);
        assert!(avg_error < 0.1, "ESN failed to converge on simple pattern");
    }
}
