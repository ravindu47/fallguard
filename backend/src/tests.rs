#[cfg(test)]
mod tests {
    // Import the functions we want to test from logic.rs
    use crate::logic::{calculate_g_force, is_fall};

    // Test 1: Check if the math for G-Force works
    #[test]
    fn test_g_force_calculation() {
        // Scenario: Device is sitting flat on a table (Z=1g)
        // sqrt(0^2 + 0^2 + 1^2) = 1.0
        let g = calculate_g_force(0.0, 0.0, 1.0);
        assert_eq!(g, 1.0);

        // Scenario: Device is falling (Weightless)
        // sqrt(0^2 + 0^2 + 0^2) = 0.0
        let g_freefall = calculate_g_force(0.0, 0.0, 0.0);
        assert_eq!(g_freefall, 0.0);
    }

    // Test 2: Check if the "Fall Threshold" works
    #[test]
    fn test_fall_trigger() {
        // Case A: High Impact (3.5G) -> Should be TRUE
        let impact_force = 3.5;
        assert!(is_fall(impact_force), "3.5G should trigger a fall alert");

        // Case B: Normal Movement (1.2G) -> Should be FALSE
        let walking_force = 1.2;
        assert!(!is_fall(walking_force), "1.2G should NOT trigger a fall alert");
    }
}