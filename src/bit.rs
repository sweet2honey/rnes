pub trait SetBit {
    fn set_bit(&mut self, bit: u8);
}

impl SetBit for u8 {
    fn set_bit(&mut self, bit: u8) {
        assert!(bit < u8::BITS as u8, "bit index must be in 0..8");
        *self |= 1 << bit;
    }
}

pub trait ClearBit {
    fn clear_bit(&mut self, bit: u8);
}

impl ClearBit for u8 {
    fn clear_bit(&mut self, bit: u8) {
        assert!(bit < u8::BITS as u8, "bit index must be in 0..8");
        *self &= !(1 << bit);
    }
}

pub trait WriteBit {
    fn write_bit(&mut self, bit: u8, is_set: bool);
}

impl WriteBit for u8 {
    fn write_bit(&mut self, bit: u8, is_set: bool) {
        assert!(bit < u8::BITS as u8, "bit index must be in 0..8");

        if is_set {
            self.set_bit(bit);
        } else {
            self.clear_bit(bit);
        }
    }
}

pub trait TestBit {
    fn test_bit(self, bit: u8) -> bool;
}

impl TestBit for u8 {
    fn test_bit(self, bit: u8) -> bool {
        assert!(bit < u8::BITS as u8, "bit index must be in 0..8");
        self & (1 << bit) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::{ClearBit, SetBit, TestBit, WriteBit};

    #[test]
    fn set_bit_sets_only_the_requested_bit() {
        let mut value = 0b0101_0001;
        value.set_bit(1);

        assert_eq!(value, 0b0101_0011);
    }

    #[test]
    fn clear_bit_clears_only_the_requested_bit() {
        let mut value = 0b0101_0011;
        value.clear_bit(1);

        assert_eq!(value, 0b0101_0001);
    }

    #[test]
    fn write_bit_sets_or_clears_only_the_requested_bit() {
        let mut value = 0b0101_0001;
        value.write_bit(1, true);
        assert_eq!(value, 0b0101_0011);

        value.write_bit(1, false);
        assert_eq!(value, 0b0101_0001);
    }

    #[test]
    fn test_bit_returns_true_when_the_bit_is_set() {
        assert!(0b0101_0011_u8.test_bit(1));
    }

    #[test]
    fn test_bit_returns_false_when_the_bit_is_clear() {
        assert!(!0b0101_0001_u8.test_bit(1));
    }
}
