use num_traits::{Num, SaturatingAdd};
use std::{cmp::Ordering, collections::BTreeSet, fmt::Display};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Unit<T: Num + SaturatingAdd> {
    Single(T),
    Range((T, T)),
}

impl<T: Num + Ord + SaturatingAdd + Copy> Unit<T> {
    fn merged(&self, other: &Self) -> Option<Unit<T>> {
        let sh = self.high();
        let ol = other.low();
        let sl = self.low();
        let oh = other.high();
        if sl < oh && sl.saturating_add(&T::one()) == *oh
            || sh < ol && sh.saturating_add(&T::one()) == *ol
        {
            Some(Unit::Range((*sl, *oh)))
        } else {
            None
        }
    }
}

impl<T: Num + SaturatingAdd> Unit<T> {
    fn low(&self) -> &T {
        match self {
            Unit::Single(l) => l,
            Unit::Range((l, _)) => l,
        }
    }

    fn high(&self) -> &T {
        match self {
            Unit::Single(h) => h,
            Unit::Range((_, h)) => h,
        }
    }
}

impl<T: Num + SaturatingAdd + Display> Display for Unit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Single(s) => write!(f, "{s}"),
            Unit::Range((t, u)) => write!(f, "{t}-{u}"),
        }
    }
}

impl<T: Num + SaturatingAdd + Ord> Ord for Unit<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<T: Num + SaturatingAdd + Ord> PartialOrd for Unit<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let sh = self.high();
        let ol = other.low();
        let sl = self.low();
        let oh = other.high();
        if sl > oh {
            Some(Ordering::Greater)
        } else if sh < ol {
            Some(Ordering::Less)
        } else {
            Some(sh.cmp(oh))
        }
    }
}

struct Ranger<T: Num + SaturatingAdd>(BTreeSet<Unit<T>>);

impl<T: Num + SaturatingAdd + Display> Display for Ranger<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for u in self.0.iter().take(self.0.len().saturating_sub(1)) {
            write!(f, "{},", u)?
        }
        if let Some(last) = self.0.iter().last() {
            write!(f, "{}", last)?
        }
        Ok(())
    }
}

impl<T: Num + SaturatingAdd + Ord + Copy + Display> Ranger<T> {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn insert(&mut self, v: T) {
        let value = Unit::Single(v);
        self.0.insert(value);
        let mut right_side = self.0.split_off(&value);
        if let Some(left) = self.0.last().copied() {
            if let Some(merged) = left.merged(&value) {
                right_side.remove(&value);
                self.0.remove(&left);
                right_side.insert(merged);
            }
        }

        while let Some(right) = right_side.iter().nth(1).copied() {
            let new_val = right_side.first().copied().unwrap();
            if let Some(m) = new_val.merged(&right) {
                right_side.remove(&right);
                right_side.remove(&new_val);
                right_side.insert(m);
            } else {
                break;
            }
        }

        self.0.extend(right_side);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{seq::SliceRandom, thread_rng};

    #[test]
    fn it_works() {
        let input_numbers: &[u8] = &[
            0, 1, 2, 4, 6, 7, 8, 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27, 28,
            29, 30, 31, 32, 33, 35, 36, 37, 38, 39,
        ];
        let mut ranger = Ranger::new();
        for num in input_numbers {
            ranger.insert(*num);
        }
        assert_eq!(ranger.to_string(), "0-2,4,6-8,11-12,14-25,27-33,35-39");
        drop(ranger);
        let input_numbers: &[u8] = &[
            33, 35, 23, 255, 254, 20, 28, 19, 18, 14, 25, 21, 253, 38, 6, 39, 27, 11, 17, 7, 12,
            128, 126, 31, 15, 32, 4, 29, 36, 22, 1, 0, 37, 30, 8, 24, 16, 2,
        ];
        let mut ranger = Ranger::new();
        for num in input_numbers {
            ranger.insert(*num);
        }
        assert_eq!(
            ranger.to_string(),
            "0-2,4,6-8,11-12,14-25,27-33,35-39,126,128,253-255"
        );
        drop(ranger);

        // Just to be extra thorough
        for _ in 0..100_000 {
            let mut myvec = input_numbers.to_vec();
            myvec.shuffle(&mut thread_rng());
            println!("{:?}", myvec);
            let mut ranger = Ranger::new();
            for num in myvec {
                ranger.insert(num);
            }

            assert_eq!(
                ranger.to_string(),
                "0-2,4,6-8,11-12,14-25,27-33,35-39,126,128,253-255"
            );
        }
    }
}
