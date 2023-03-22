#![no_std]
mod parse;
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
    h: Option<T>,
}

enum Merger<T> {
    Merged,
    NotMerged(T),
}

impl<T: Num + Ord + SaturatingSub> Unit<T> {
    fn merged(&mut self, other: Self) -> Merger<Unit<T>> {
        if other
            .l
            .saturating_sub(self.h.as_ref().unwrap_or(&self.l))
            .is_one()
        {
            self.h = other.h.or(Some(other.l));
            Merger::Merged
        } else {
            Merger::NotMerged(other)
        }
    }
}

impl<T: Ord> Ord for Unit<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { self.partial_cmp(other).unwrap_unchecked() }
    }
}

impl<T: Ord> PartialOrd for Unit<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let sh = self.h.as_ref().unwrap_or(&self.l);
        let oh = other.h.as_ref().unwrap_or(&other.l);
        if &self.l > oh {
            Some(Ordering::Greater)
        } else if sh < &other.l {
            Some(Ordering::Less)
        } else {
            Some(sh.cmp(oh))
        }
    }
}

impl<T: Eq + Display> Display for Unit<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sh = self.h.as_ref().unwrap_or(&self.l);
        if &self.l == sh {
            write!(f, "{}", self.l)
        } else {
            write!(f, "{}-{}", self.l, sh)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Ranger<T>(BTreeSet<Unit<T>>);

impl<T: Eq + Display> Display for Ranger<T> {
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

/// Pops the element immediately before the specified value
pub fn pop_before<K: Ord>(set: &mut BTreeSet<K>, value: &K) -> Option<K> {
    let key_ref = {
        if let Some(key_ref) = set.range(..=value).next_back() {
            /* must hide the origin of this borrow ... */
            unsafe { &*(key_ref as *const _) }
        } else {
            return None;
        }
    };
    /* ... so that we may be able to mutably borrow the set here
    despite key_ref existence */
    set.take(key_ref)
}

/// Pops the element immediately after the specified value
pub fn pop_after<K: Ord>(set: &mut BTreeSet<K>, value: &K) -> Option<K> {
    let key_ref = {
        if let Some(key_ref) = set.range(value..).next() {
            /* must hide the origin of this borrow ... */
            unsafe { &*(key_ref as *const _) }
        } else {
            return None;
        }
    };
    /* ... so that we may be able to mutably borrow the set here
    despite key_ref existence */
    set.take(key_ref)
}

/// Pops the element equal to the value specified
pub fn pop<K>(set: &mut BTreeSet<K>, value: &K) -> Option<K>
where
    K: Ord,
{
    let key_ref = {
        if let Some(key_ref) = set.range(..=value).next() {
            /* must hide the origin of this borrow ... */
            unsafe { &*(key_ref as *const _) }
        } else {
            return None;
        }
    };
    /* ... so that we may be able to mutably borrow the set here
    despite key_ref existence */
    set.take(key_ref)
}

impl<T: Num + SaturatingSub + Ord + Display> Ranger<T> {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }
    pub fn contains(&self, value: &T) -> bool {
        let mut contained = false;
        let v = unsafe {core::ptr::read(value)};
        let u = Unit { l: v, h: None };
        if let Some(v) = self.0.range(&u..).next() {
            if v.l <= u.l && &u.l <= v.h.as_ref().unwrap_or(&v.l) {
                contained = true;
            }
        }
        core::mem::forget(u.l);
        contained
    }
    pub fn insert(&mut self, value: T) -> bool {
        if self.contains(&value) {
            return false
        }
        let u = Unit {
            l: value,
            h: None,
        };
        let v = if let Some(mut low) = pop_before(&mut self.0, &u) {
            match low.merged(u) {
                Merger::Merged => low,
                Merger::NotMerged(value) => {
                    self.0.insert(low);
                    value
                }
            }
        } else {
            u
        };
        let mut holster = Some(v);
        while let Some(mut low) = holster.take() {
            if let Some(high) = pop_after(&mut self.0, &low) {
                match low.merged(high) {
                    Merger::Merged => {
                        holster = Some(low);
                    }
                    Merger::NotMerged(high) => {
                        self.0.insert(low);
                        self.0.insert(high);
                    }
                }
            } else {
                self.0.insert(low);
            }
        }
        true
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
            0, 1, 2, 4, 6, 7, 8, 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 0, 22, 23, 24, 25, 27, 28,
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
            -1, 33, 35, 23, 20, -128, 28, 0, 19, 18, 14, 25, 21, 127, 38, 6, 39, 27, 11, 17, 7, 12,
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
