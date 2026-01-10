
use reflex::governor::sentinel::{Sentinel, VitalityStatus};
use reflex::governor::ignition::{IgnitionSequence, IgnitionState};
use reflex::governor::rebalancer::Rebalancer;
use std::time::Duration;

#[test]
fn test_sentinel_jitter_detection() {
    let mut sentinel = Sentinel::new();

    // 1. Initial State (Optimal)
    assert_eq!(sentinel.status, VitalityStatus::Optimal);

    // 2. Simulate Perfect Ticks (0 Jitter)
    // We override internal state or just call tick() rapidly?
    // `tick()` measures elapsed time. We can't easily mock time in a unit test without refactoring Sentinel to accept a Clock trait.
    // However, the `tick()` function calculates `current_latency_us` based on `Instant::now()`.
    // In a tight loop, it should be very small and consistent.

    for _ in 0..105 {
        sentinel.tick();
        // Busy wait to create some duration?
        // Actually, just calling tick() is fine.
    }

    // Should be Optimal (Jitter ~0)
    assert_eq!(sentinel.status, VitalityStatus::Optimal);

    // Note: To test Degraded/Critical, we'd need to mock time or sleep randomly, which makes tests flaky.
    // We assume the logic `if jitter > threshold` works if we could inject jitter.
}

#[test]
fn test_ignition_state_transitions() {
    let mut ignition = IgnitionSequence::new();
    let mut sentinel = Sentinel::new();

    // 1. Initial State
    assert_eq!(ignition.state, IgnitionState::Hibernation);

    // 2. Launch
    ignition.initiate_launch();
    assert_eq!(ignition.state, IgnitionState::HardwareCheck);

    // 3. Hardware Check (Requires Sentinel Stable for 300s)
    // We can't wait 300s. We'd need to mock `sentinel.is_stable_for`.
    // Since we can't mock without traits/mocking lib easily here, we will verify that
    // it *doesn't* advance if sentinel is fresh (stable_for returns false usually unless we fake time).

    ignition.update(&sentinel, true);
    assert_eq!(ignition.state, IgnitionState::HardwareCheck);
}

#[test]
fn test_rebalancer_fidelity_logic() {
    let mut rebalancer = Rebalancer::new(10000.0);

    // 1. Initial Fidelity
    assert_eq!(rebalancer.fidelity, 1.0);
    assert_eq!(rebalancer.get_safe_size(1.0), 1.0);

    // 2. Punish
    rebalancer.punish_nullification(); // 0.95
    rebalancer.punish_nullification(); // 0.90
    assert!((rebalancer.fidelity - 0.90).abs() < 0.001);

    // 3. Size Scaling
    assert!((rebalancer.get_safe_size(100.0) - 90.0).abs() < 0.001);

    // 4. Critical Fidelity
    for _ in 0..10 { rebalancer.punish_nullification(); } // -0.50 -> 0.40
    assert!(rebalancer.fidelity < 0.5);
    assert_eq!(rebalancer.get_safe_size(100.0), 0.0); // Locked

    // 5. Reward
    rebalancer.reward_success(); // +0.01 -> 0.41
    assert!((rebalancer.fidelity - 0.41).abs() < 0.001);
}

#[test]
fn test_rebalancer_omega_protocol() {
    let mut rebalancer = Rebalancer::new(10000.0);

    // 1. Safe Drawdown
    assert_eq!(rebalancer.check_omega(9000.0), false); // 10% DD

    // 2. Critical Drawdown (>15%)
    assert_eq!(rebalancer.check_omega(8400.0), true); // 16% DD
}
