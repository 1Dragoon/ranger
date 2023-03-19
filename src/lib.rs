#![no_std]
extern crate alloc;
use alloc::collections::BTreeSet;
use core::{
    cmp::Ordering,
    fmt::{self, Display},
};
use num_traits::{Num, SaturatingSub};

#[derive(Clone, Eq, PartialEq, Debug)]
struct Unit<T> {
    l: T,
    h: T,
}

enum Merger<T> {
    Merged(T),
    NotMerged(T, T),
}

impl<T: Num + Ord + SaturatingSub> Unit<T> {
    fn merged(self, other: Self) -> Merger<Unit<T>> {
        if other.l.saturating_sub(&self.h).is_one() {
            Merger::Merged(Unit {
                l: self.l,
                h: other.h,
            })
        } else {
            Merger::NotMerged(self, other)
        }
    }
    // fn singleton(value: T) -> Self {
    //     Self{l: value, h: Cow::Borrowed}
    // }
}

impl<T: Num + SaturatingSub + Ord> Ord for Unit<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { self.partial_cmp(other).unwrap_unchecked() }
    }
}

impl<T: Num + SaturatingSub + Ord> PartialOrd for Unit<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.l > other.h {
            Some(Ordering::Greater)
        } else if self.h < other.l {
            Some(Ordering::Less)
        } else {
            Some(self.h.cmp(&other.h))
        }
    }
}

impl<T: Num + Display> Display for Unit<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.l == self.h {
            write!(f, "{}", self.l)
        } else {
            write!(f, "{}-{}", self.l, self.h)
        }
    }
}

#[derive(Default)]
pub struct Ranger<T: Num>(BTreeSet<Unit<T>>);

impl<T: Num + Display> Display for Ranger<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for u in self.0.iter().take(self.0.len().saturating_sub(1)) {
            write!(f, "{},", u)?
        }
        if let Some(last) = self.0.iter().last() {
            write!(f, "{}", last)?
        }
        Ok(())
    }
}

impl<T: Num + SaturatingSub + Ord + Clone> Ranger<T> {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }
    pub fn insert(&mut self, value: T) {
        let v = Unit{l: value.clone(), h: value};
        self.merge_at(v.clone());
        while self.0.len() > 1 && self.merge_from(&v) {}
    }
    fn merge_at(&mut self, value: Unit<T>) {
        let mut high_side = self.0.split_off(&value);
        high_side.insert(value);
        if let Some(low) = self.0.pop_last() {
            if let Some(high) = high_side.pop_first() {
                match low.merged(high) {
                    Merger::Merged(merged) => {
                        self.0.insert(merged);
                    },
                    Merger::NotMerged(low, high) => {
                        self.0.insert(low);
                        self.0.insert(high);
                    },
                }
            } else {
                self.0.insert(low);
            }
        }
        self.0.extend(high_side);
    }
    fn merge_from(&mut self, value: &Unit<T>) -> bool {
        let mut merge_done = false;
        let mut high_set = self.0.split_off(value);
        if high_set.len() > 1 {
            let (low, high) = unsafe {
                (
                    high_set.pop_first().unwrap_unchecked(),
                    high_set.pop_first().unwrap_unchecked(),
                )
            };
            match low.merged(high) {
                Merger::Merged(merged) => {
                    self.0.insert(merged);
                    merge_done = true;
                }
                Merger::NotMerged(low, high) => {
                    self.0.insert(low);
                    self.0.insert(high);
                }
            }
        }
        self.0.extend(high_set);
        merge_done
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use libc_print::std_name::println;
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
