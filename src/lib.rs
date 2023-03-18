use itertools::Itertools;
use num_traits::{Num, SaturatingSub};
use std::{cmp::Ordering, collections::BTreeSet, fmt::Display};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Unit<T: Num + SaturatingSub> {
    Singleton(T),
    Range((T, T)),
}

impl<T: Num + Ord + SaturatingSub + Copy> Unit<T> {
    fn merged(&self, other: &Self) -> Option<Unit<T>> {
        if other.low().saturating_sub(self.high()).is_one() {
            Some(Unit::Range((*self.low(), *other.high())))
        } else {
            None
        }
    }

    fn low(&self) -> &T {
        match self {
            Unit::Singleton(l) => l,
            Unit::Range((l, _)) => l,
        }
    }

    fn high(&self) -> &T {
        match self {
            Unit::Singleton(h) => h,
            Unit::Range((_, h)) => h,
        }
    }
}

impl<T: Num + SaturatingSub + Ord + Copy> Ord for Unit<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { self.partial_cmp(other).unwrap_unchecked() }
    }
}

impl<T: Num + SaturatingSub + Ord + Copy> PartialOrd for Unit<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.low() > other.high() {
            Some(Ordering::Greater)
        } else if self.high() < other.low() {
            Some(Ordering::Less)
        } else {
            Some(self.high().cmp(other.high()))
        }
    }
}

impl<T: Num + SaturatingSub + Display> Display for Unit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Singleton(s) => write!(f, "{s}"),
            Unit::Range((t, u)) => write!(f, "{t}-{u}"),
        }
    }
}

#[derive(Default)]
pub struct Ranger<T: Num + SaturatingSub>(BTreeSet<Unit<T>>);

impl<T: Num + SaturatingSub + Display> Display for Ranger<T> {
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

impl<T: Num + SaturatingSub + Ord + Copy> Ranger<T> {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn insert(&mut self, v: T) {
        let value = Unit::Singleton(v);
        self.0.insert(value);
        let mut high_side = self.0.split_off(&value);
        if let Some(low) = self.0.last().copied() {
            if let Some(merged) = low.merged(&value) {
                high_side.remove(&value);
                self.0.remove(&low);
                high_side.insert(merged);
            }
        }

        while let Some((m, l, h)) = high_side
            .iter()
            .copied()
            .next_tuple()
            .and_then(|(l, h)| l.merged(&h).map(|m| (m, l, h)))
        {
            high_side.remove(&l);
            high_side.remove(&h);
            high_side.insert(m);
        }
        self.0.extend(high_side);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{seq::SliceRandom, thread_rng};

    #[test]
    fn it_works() {
        let input_numbers: &mut [u8] = &mut [
            0, 1, 2, 4, 6, 7, 8, 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27, 28,
            29, 30, 31, 32, 33, 35, 36, 37, 38, 39,
        ];
        let mut ranger = Ranger::new();
        for num in input_numbers.iter() {
            ranger.insert(*num);
        }
        assert_eq!(ranger.to_string(), "0-2,4,6-8,11-12,14-25,27-33,35-39");
        drop(ranger);
        input_numbers.reverse();
        let mut ranger = Ranger::new();
        for num in input_numbers {
            ranger.insert(*num);
        }
        assert_eq!(ranger.to_string(), "0-2,4,6-8,11-12,14-25,27-33,35-39");
        drop(ranger);
        let input_numbers: &[i8] = &[
            -1, 33, 35, 23, 20, -128, 28, 19, 18, 14, 25, 21, 127, 38, 6, 39, 27, 11, 17, 7, 12,
            126, -126, 31, 15, 32, 4, 29, 36, 22, 1, 0, 37, 30, 8, 24, 16, 2, -127, 125,
        ];
        let mut ranger = Ranger::new();
        for num in input_numbers {
            ranger.insert(*num);
        }
        assert_eq!(
            ranger.to_string(),
            "-128--126,-1-2,4,6-8,11-12,14-25,27-33,35-39,125-127"
        );
        println!("{:?} -> {ranger}", input_numbers);
        drop(ranger);

        // Rule out edge cases
        for _ in 0..10_000 {
            let mut myvec = input_numbers.to_vec();
            myvec.shuffle(&mut thread_rng());
            // println!("{:?}", myvec);
            let mut ranger = Ranger::new();
            for num in myvec {
                ranger.insert(num);
            }

            assert_eq!(
                ranger.to_string(),
                "-128--126,-1-2,4,6-8,11-12,14-25,27-33,35-39,125-127"
            );
        }
    }
}
