use num_traits::{Num, SaturatingAdd};
use std::{cmp::Ordering, collections::BTreeSet, fmt::Display};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Unit<T: Num + SaturatingAdd> {
    Single(T),
    Range((T, T)), // lower bound and upper bound
}

impl<T: Num + Ord + SaturatingAdd + Copy + Display> Unit<T> {
    fn umm(&self, other: &Self) -> Option<Unit<T>> {
        if self == other {
            return None;
        }
        let left = self.min(other);
        let right = self.max(other);
        match left {
            Unit::Single(l) => match right {
                Unit::Single(r) => {
                    if l < r && l.saturating_add(&T::one()) == *r {
                        Some(Unit::Range((*l, *r)))
                    } else {
                        None
                    }
                }
                Unit::Range((rl, r)) => {
                    if l < rl {
                        if l.saturating_add(&T::one()) == *rl {
                            Some(Unit::Range((*l, *r)))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            },
            Unit::Range((ll, lr)) => match other {
                Unit::Single(r) => {
                    if lr < r {
                        if lr.saturating_add(&T::one()) == *r {
                            Some(Unit::Range((*ll, *r)))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Unit::Range((rl, rr)) => {
                    if lr < rl && lr.saturating_add(&T::one()) == *rl || ll <= rr && rl <= lr {
                        Some(Unit::Range((*ll, *rr)))
                    } else {
                        None
                    }
                }
            },
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
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

impl<T: Num + SaturatingAdd + Ord> PartialOrd for Unit<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            Unit::Single(s) => match other {
                Unit::Single(o) => Some(s.cmp(o)),
                Unit::Range((ol, oh)) => {
                    if s < ol {
                        Some(Ordering::Less)
                    } else if s > oh {
                        Some(Ordering::Greater)
                    } else {
                        None
                    }
                }
            },
            Unit::Range((sl, sh)) => match other {
                Unit::Single(o) => {
                    if sl > o {
                        Some(Ordering::Greater)
                    } else if sh < o {
                        Some(Ordering::Less)
                    } else {
                        None
                    }
                }
                Unit::Range((ol, oh)) => {
                    if sl == ol && sh == oh {
                        Some(Ordering::Equal)
                    } else if sl > oh {
                        Some(Ordering::Greater)
                    } else if sh < ol {
                        Some(Ordering::Less)
                    } else {
                        None
                    }
                }
            },
        }
    }
}

#[derive(Debug)]
struct Ranger<T: Num + SaturatingAdd> {
    set: BTreeSet<Unit<T>>,
}

impl<T: Num + SaturatingAdd + Display> Display for Ranger<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for u in self.set.iter().take(self.set.len().saturating_sub(1)) {
            write!(f, "{},", u)?
        }
        if let Some(last) = self.set.iter().last() {
            write!(f, "{}", last)?
        }
        Ok(())
    }
}

impl<T: Num + SaturatingAdd + Ord + Copy + Display> Ranger<T> {
    fn put(&mut self, v: T) {
        let value = Unit::Single(v);
        self.set.insert(value);
        let mut right_side = self.set.split_off(&value);
        if let Some(left) = self.set.last().copied() {
            if let Some(merged) = left.umm(&value) {
                    right_side.remove(&value);
                    self.set.remove(&left);
                    right_side.insert(merged);
                }
            }

        while let Some(new_right) = right_side.iter().nth(1).copied() {
            let new_val = right_side.first().copied().unwrap();
            if let Some(m) = new_val.umm(&new_right) {
                right_side.remove(&new_right);
                right_side.remove(&new_val);
                right_side.insert(m);
            } else {
                break;
            }
        }

        self.set.extend(right_side);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use rand::{seq::SliceRandom, thread_rng};

    use super::*;

    #[test]
    fn it_works() {
        // Expected: 0-2,4,6-8,11-12,14-25,27-33,35-39
        let input_numbers: &[u16] = &[
            0, 1, 2, 4, 6, 7, 8, 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27, 28,
            29, 30, 31, 32, 33, 35, 36, 37, 38, 39,
        ];
        let mut ranger = Ranger {
            set: BTreeSet::new(),
        };
        for num in input_numbers {
            ranger.put(*num);
        }
        println!("{}", ranger);
        assert!(ranger.to_string() == "0-2,4,6-8,11-12,14-25,27-33,35-39");
        drop(ranger);
        println!(
            "
        
        Tada!

        "
        );
        let input_numbers: &[u16] = &[
            33, 35, 23, 20, 28, 19, 18, 14, 25, 21, 38, 6, 39, 27, 11, 17, 7, 12, 31, 15, 32, 4,
            29, 36, 22, 1, 0, 37, 30, 8, 24, 16, 2, 65535, 65534, 65531, 65533, 65532, 65530
        ];
        let mut ranger = Ranger {
            set: BTreeSet::new(),
        };
        for num in input_numbers {
            ranger.put(*num);
        }
        println!(
            "
        
        
        {}",
            ranger
        );
        assert!(ranger.to_string() == "0-2,4,6-8,11-12,14-25,27-33,35-39,65530-65535");
        drop(ranger);

        for _ in 0..100 {
            let mut myvec = input_numbers.to_vec();
            myvec.shuffle(&mut thread_rng());
            // println!("{:?}", myvec);
            let mut ranger = Ranger {
                set: BTreeSet::new(),
            };
            println!("{:?}", myvec);
            for num in myvec {
                ranger.put(num);
            }
            assert!(ranger.to_string() == "0-2,4,6-8,11-12,14-25,27-33,35-39,65530-65535");
        }
    }
}

// 0-2,8,4,6-8,11,12,14-25,27-33,35-39
// 0-2,6-8,11-12,14-25,27-33,35-39,4
