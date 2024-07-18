pub trait Digits {
    fn digits(&self) -> impl Iterator<Item=u8>;
}

impl Digits for u8 {
    fn digits(&self) -> impl Iterator<Item=u8> {
        let mut num = *self;
        const DIGIT_COUNT: usize = (u8::MAX.ilog10() + 1) as usize;
        let mut digits: [u8; DIGIT_COUNT] = [0; DIGIT_COUNT];
        for i in 0..DIGIT_COUNT {
            digits[DIGIT_COUNT - i - 1] = num % 10;
            num /= 10;
        }
        return digits.into_iter();
    }
}

pub trait NextTo {
    fn is_next_to(&self, other: &Self) -> bool;
}

impl NextTo for (u16, u16) {
    fn is_next_to(&self, other: &Self) -> bool {
        let (tx, ty) = *self;
        let (ox, oy) = *other;
        let dx = tx.abs_diff(ox);
        let dy = ty.abs_diff(oy);

        return (dx == 1 && dy == 0) || (dy == 1 && dx == 0);
    }
}
