use std::collections::HashMap;

/// Calculates the arithmetic mean (average) of a slice of numeric values.
///
/// # Arguments
/// * `values` - A slice of values to average
///
/// # Returns
/// The arithmetic mean as f64, or 0.0 if the slice is empty
pub fn mean(values: &[usize]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let sum: usize = values.iter().sum();
    sum as f64 / values.len() as f64
}

/// Calculates the median value of a slice of numeric values.
///
/// # Arguments
/// * `values` - A slice of values to find the median of
///
/// # Returns
/// The median as f64, or 0.0 if the slice is empty
pub fn median(values: &[usize]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted_values = values.to_vec();
    sorted_values.sort_unstable();

    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        // Even number of elements - average the two middle values
        (sorted_values[mid - 1] as f64 + sorted_values[mid] as f64) / 2.0
    } else {
        // Odd number of elements - return the middle value
        sorted_values[mid] as f64
    }
}

/// Calculates the standard deviation of a slice of numeric values.
///
/// # Arguments
/// * `values` - A slice of values to calculate the standard deviation of
///
/// # Returns
/// The standard deviation as f64, or 0.0 if the slice is empty or has only one element
pub fn standard_deviation(values: &[usize]) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let avg = mean(values);
    let variance = values
        .iter()
        .map(|&value| {
            let diff = avg - (value as f64);
            diff * diff
        })
        .sum::<f64>()
        / (values.len() - 1) as f64;

    variance.sqrt()
}

/// Calculates the coefficient of variation (CV) which is the ratio of
/// the standard deviation to the mean. It's useful for comparing relative variability.
///
/// # Arguments
/// * `values` - A slice of values to calculate the CV of
///
/// # Returns
/// The coefficient of variation as a percentage (0-100), or 0.0 if the mean is 0
pub fn coefficient_of_variation(values: &[usize]) -> f64 {
    let m = mean(values);
    if m == 0.0 {
        return 0.0;
    }

    (standard_deviation(values) / m) * 100.0
}

/// Performs a simple statistical significance test (two-sample t-test)
/// to determine if the difference between two sets of measurements is significant.
///
/// # Arguments
/// * `group1` - First set of measurements
/// * `group2` - Second set of measurements
/// * `alpha` - Significance level (default is often 0.05)
///
/// # Returns
/// A tuple containing:
/// - p-value approximation
/// - boolean indicating if the difference is statistically significant
/// - percentage difference between means
pub fn compare_measurements(group1: &[usize], group2: &[usize], alpha: f64) -> (f64, bool, f64) {
    if group1.is_empty() || group2.is_empty() {
        return (1.0, false, 0.0);
    }

    let mean1 = mean(group1);
    let mean2 = mean(group2);

    // Calculate percentage difference
    let percent_diff = if mean1 != 0.0 && mean2 != 0.0 {
        ((mean2 - mean1) / mean1) * 100.0
    } else {
        0.0
    };

    // Calculate t-statistic
    let n1 = group1.len() as f64;
    let n2 = group2.len() as f64;

    let var1 = standard_deviation(group1).powi(2);
    let var2 = standard_deviation(group2).powi(2);

    // Prevent division by zero
    if var1 == 0.0 && var2 == 0.0 {
        return (1.0, false, percent_diff);
    }

    // Calculate t-statistic using Welch's t-test
    let t_stat = (mean1 - mean2).abs() / ((var1 / n1 + var2 / n2).sqrt());

    // Approximate degrees of freedom using Welch-Satterthwaite equation
    let df = (var1 / n1 + var2 / n2).powi(2)
        / ((var1.powi(2) / (n1.powi(2) * (n1 - 1.0))) + (var2.powi(2) / (n2.powi(2) * (n2 - 1.0))));

    // Simple p-value approximation
    // This is a very rough approximation
    let p_value = 1.0 / (1.0 + t_stat * (0.196854 + t_stat * (0.115194 + t_stat * 0.000344)));

    (p_value, p_value < alpha, percent_diff)
}

/// Formats byte size to a human-readable string with appropriate unit
///
/// # Arguments
/// * `bytes` - Number of bytes
///
/// # Returns
/// A formatted string representing the size with appropriate unit (B, KB, MB, GB)
pub fn format_bytes(bytes: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes_f64 = bytes as f64;

    if bytes_f64 < KB {
        format!("{:.0} B", bytes_f64)
    } else if bytes_f64 < MB {
        format!("{:.2} KB", bytes_f64 / KB)
    } else if bytes_f64 < GB {
        format!("{:.2} MB", bytes_f64 / MB)
    } else {
        format!("{:.2} GB", bytes_f64 / GB)
    }
}

/// Summarizes a set of measurements and returns a HashMap with various statistics
///
/// # Arguments
/// * `measurements` - A slice of numeric measurements
///
/// # Returns
/// A HashMap containing various statistics of the measurements
pub fn summarize_statistics(measurements: &[usize]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();

    stats.insert("n".to_string(), measurements.len() as f64);
    stats.insert("mean".to_string(), mean(measurements));
    stats.insert("median".to_string(), median(measurements));
    stats.insert("std_dev".to_string(), standard_deviation(measurements));
    stats.insert("cv".to_string(), coefficient_of_variation(measurements));

    if !measurements.is_empty() {
        let min = *measurements.iter().min().unwrap_or(&0) as f64;
        let max = *measurements.iter().max().unwrap_or(&0) as f64;

        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("range".to_string(), max - min);
    }

    stats
}
