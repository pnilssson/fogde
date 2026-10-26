pub fn real(value: u32) -> f32 {
    let high = u16::try_from(value >> 16).unwrap_or(u16::MAX);
    let low = u16::try_from(value & 0xffff).unwrap_or(u16::MAX);
    f32::from(high) * 65_536. + f32::from(low)
}

pub fn real_usize(value: usize) -> f32 {
    real(u32::try_from(value).unwrap_or(u32::MAX))
}

pub fn floor(value: f32) -> u32 {
    floor64(f64::from(value))
}

pub fn floor64(value: f64) -> u32 {
    if value.is_nan() || value < 1. {
        return 0;
    }
    if value >= f64::from(u32::MAX) {
        return u32::MAX;
    }

    let (mut low, mut high) = (0u32, u32::MAX);
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if f64::from(middle) <= value {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    low
}

pub fn floor_u16(value: f32) -> u16 {
    u16::try_from(floor(value)).unwrap_or(u16::MAX)
}

pub fn round(value: f32) -> u32 {
    floor(value + 0.5)
}

pub fn index(value: f32) -> usize {
    usize::try_from(floor(value)).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floors_like_a_cast_but_saturates() {
        assert_eq!(floor(0.), 0);
        assert_eq!(floor(0.99), 0);
        assert_eq!(floor(7.5), 7);
        assert_eq!(floor(-3.), 0);
        assert_eq!(floor(f32::NAN), 0);
        assert_eq!(floor(1e12), u32::MAX);
        assert_eq!(round(2.5), 3);
        assert_eq!(floor_u16(70_000.), u16::MAX);
    }

    #[test]
    fn converts_counts_to_floats() {
        assert!((real(70_000) - 70_000.).abs() < f32::EPSILON);
        assert!((real(12) - 12.).abs() < f32::EPSILON);
    }
}
