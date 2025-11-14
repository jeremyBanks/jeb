pub use static_assertions::const_assert;

#[macro_export]
macro_rules! r#noop {
    ($($x:expr $(;)?)+) => {
        $(
            const _: [(); {
                {
                    $x
                };
                0
            }] = [];
        )+
    };
}

pub const fn eq_usize(value: usize, calculation: usize) -> usize {
    if value != calculation {
        panic!("calculation did not match actual value");
    }

    value
}

pub const fn eq_bytes(expected: &[u8], calculated: &[u8]) -> bool {
    if expected.len() != calculated.len() {
        panic!("calculated value had different length than expected value");
    }

    let mut index = 0;
    while index < expected.len() {
        if expected[index] != calculated[index] {
            panic!("calculated value did not match expected value");
        }
        index += 1;
    }

    true
}

pub const fn div_exact(dividend: usize, divisor: usize) -> usize {
    if dividend % divisor != 0 {
        panic!("remainder in div_exact");
    }

    dividend / divisor
}
pub const fn pow(base: usize, exponent: usize) -> usize {
    let mut result: usize = 1;

    let mut iterations: usize = 0;
    while iterations < exponent {
        result *= base;
        iterations += 1;
    }

    result
}
