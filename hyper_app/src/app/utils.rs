#[cfg(target_arch = "wasm32")]
pub(crate) fn file_save(content: &str) {
    use wasm_bindgen::prelude::wasm_bindgen;
    #[wasm_bindgen]
    extern "C" {
        fn alert(s: &str);
    }
    eframe::web_sys::console::log_1(&content.into());
    alert("(WIP) Look a the debug console to copy the file :)");
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn file_save(_content: &str) {
    // TODO
    println!("TODO save file")
}

pub(crate) struct SecFmt(pub f64);

impl From<f64> for SecFmt {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for SecFmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.precision()
        let x = self.0;
        let (t, n) = if x > 60.0 {
            let n = if f.alternate() { "minutes" } else { "m" };
            (x / 60.0, n)
        } else if x == 0.0 {
            let n = if f.alternate() { "seconds" } else { "s" };
            (x, n)
        } else if x < 0.00_000_000_001 {
            let n = if f.alternate() { "pico seconds" } else { "ps" };
            (x * 1_000_000_000_000., n)
        } else if x < 0.00_000_001 {
            let n = if f.alternate() { "nano seconds" } else { "ns" };
            (x * 1_000_000_000., n)
        } else if x < 0.00_001 {
            let n = if f.alternate() { "micro seconds" } else { "us" };
            (x * 1_000_000., n)
        } else if x < 1.0 {
            let n = if f.alternate() { "milli seconds" } else { "ms" };
            (x * 1_000., n)
        } else {
            let n = if f.alternate() { "seconds" } else { "s" };
            (x, n)
        };
        if t == 0.0 {
            write!(f, "{:.1} {}", t, n)
        } else if let Some(prec) = f.precision() {
            write!(f, "{} {}", round_to_significant_digits3(t, prec), n)
        } else {
            write!(f, "{} {}", t, n)
        }
    }
}

pub fn round_to_significant_digits3(number: f64, significant_digits: usize) -> String {
    if number == 0.0 {
        return format!("{:.*}", significant_digits, number);
    }
    let abs = number.abs();
    let d = if abs == 1.0 {
        1.0
    } else {
        (abs.log10().ceil()).max(0.0)
    };
    let power = significant_digits - d as usize;

    let magnitude = 10.0_f64.powi(power as i32);
    let shifted = number * magnitude;
    let rounded_number = shifted.round();
    let unshifted = rounded_number as f64 / magnitude;
    dbg!(
        number,
        (number.abs() + 0.000001).log10().ceil(),
        significant_digits,
        power,
        d
    );
    format!("{:.*}", power, unshifted)
}

#[test]
fn seconde_formating_test() {
    assert_eq!(format!("{:.4}", SecFmt(0.0)), "0.0 s");
    assert_eq!(format!("{:.3}", SecFmt(1.0 / 1000.0)), "1.00 ms");
    assert_eq!(format!("{:.3}", SecFmt(1.0 / 1000.0 / 1000.0)), "1.00 us");
    assert_eq!(format!("{:.4}", SecFmt(0.00_000_000_1)), "1.000 ns");
    assert_eq!(format!("{:.4}", SecFmt(0.00_000_000_000_1)), "1.000 ps");
    assert_eq!(format!("{:.2}", SecFmt(0.0000000012)), "1.2 ns");
    assert_eq!(format!("{:.4}", SecFmt(10.43333)), "10.43 s");
    assert_eq!(format!("{:.3}", SecFmt(10.43333)), "10.4 s");
    assert_eq!(format!("{:.2}", SecFmt(10.43333)), "10 s");
    assert_eq!(format!("{:3e}", 10.43333), "1.043333e1");
}
