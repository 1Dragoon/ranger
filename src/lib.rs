use std::{cmp::Ordering, collections::BTreeSet, fmt::Display};
use num_traits::{Num, SaturatingAdd, SaturatingSub};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Unit<T> {
    Single(T),
    Range((T, T)), // lower bound and upper bound
}

impl<T: Display> Display for Unit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Single(s) => write!(f, "{s}"),
            Unit::Range((t, u)) => write!(f, "{t}-{u}"),
        }
    }
}

/// Returns true if the two were equal or adjacent, and if adjacent then it also expands self
impl<T: Num + SaturatingSub + SaturatingAdd + Ord + Eq + Copy + Display> Unit<T> {
    fn combine(&mut self, other: &Self) -> bool {
        if self == other {
            // Singletons falling within another range are considered equal
            return true;
        }
        match self {
            Unit::Single(s) => match other {
                Unit::Single(o) => {
                    let low = (*s).min(*o);
                    let high = (*s).max(*o);
                    if high.saturating_sub(&T::one()) == low {
                        // Low is one less than high, so convert to a range
                        *self = Unit::Range((low, high));
                        true
                    } else {
                        false
                    }
                }
                Unit::Range((ol, oh)) => {
                    if ((*ol).saturating_sub(&T::one())) == *s {
                        // Self is one less than other lower, so modify the range with self as the lower and other high as the higher
                        *self = Unit::Range((*s, *oh));
                        true
                    } else if ((*oh).saturating_add(&T::one())) == *s {
                        // Self is one more than other higher, so modify the range with self as the higher and the other low as the lower
                        *self = Unit::Range((*ol, *s));
                        true
                    } else {
                        false
                    }
                }
            },
            Unit::Range((sl, sh)) => match other {
                Unit::Single(o) => {
                    if *sh == (*o).saturating_sub(&T::one()) {
                        // Self higher is one less than other, so modify the range with self lower as the lower and other as the higher
                        *self = Unit::Range((*sl, *o));
                        true
                    } else if *sl == (*o).saturating_add(&T::one()) {
                        // Self lower is one more than other, so modify the range with self as the higher and the other low as the lower
                        *self = Unit::Range((*o, *sh));
                        true
                    } else {
                        false
                    }
                }
                Unit::Range((ol, oh)) => {
                    // Determine if they overlap. If they do, then combine them
                    if *sl <= *oh && ol <= sh || (*sh + T::one()) == *ol || (*oh + T::one()) == *sl
                    {
                        let lower = (*sl).min(*ol);
                        let upper = (*sh).max(*oh);
                        *self = Unit::Range((lower, upper));
                        true
                    } else {
                        false
                    }
                }
            },
        }
    }
}

impl<T: Ord> PartialOrd for Unit<T> {
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
                        Some(Ordering::Equal)
                    }
                }
            },
            Unit::Range((sl, sh)) => match other {
                Unit::Single(o) => {
                    if o < sl {
                        Some(Ordering::Less)
                    } else if o > sh {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Equal)
                    }
                }
                Unit::Range((ol, oh)) => {
                    if sl == ol && sh == oh {
                        Some(Ordering::Equal)
                    } else if sl > oh {
                        Some(Ordering::Greater)
                    } else {
                        // Anything not exactly equal bounds or having the entire boundary higher is considered less, including anything that overlaps
                        Some(Ordering::Less)
                    }
                }
            },
        }
    }
}

impl<T: Ord> Ord for Unit<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug)]
struct Ranger<T> {
    set: BTreeSet<Unit<T>>,
}

impl<T: Display> Display for Ranger<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for u in self.set.iter().take(self.set.len()-1) {
            write!(f, "{},", u)?
        }
        write!(f, "{}", self.set.iter().last().unwrap())
    }
}

impl<T: Num + SaturatingSub + SaturatingAdd + Ord + Eq + Copy + std::fmt::Display> Ranger<T> {
    fn put(&mut self, v: T) {
        let mut value = Unit::Single(v);
        if self.set.is_empty() {
            self.set.insert(value);
            return;
        }
        if self.set.contains(&value) {
            return;
        }
        let mut remove = None;
        for i in &self.set {
            if value.combine(i) {
                remove = Some(*i);
                break;
            }
        }
        if let Some(r) = remove {
            self.set.remove(&r);
        }
        self.set.insert(value);
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn it_works() {
        let input_numbers: &[u16] = &[
            0, 1, 2, 4, 6, 7, 8, 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27, 28, 29,
            30, 31, 32, 33, 35, 36, 37, 38, 39,
        ];
        let mut ranger = Ranger{
            set: BTreeSet::new(),
        };
        for num in input_numbers {
            ranger.put(*num);
        }
        println!("{}", ranger)
    }
}

// 0-2,8,4,6-8,11,12,14-25,27-33,35-39
// 0-2,6-8,11-12,14-25,27-33,35-39,4