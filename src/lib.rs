use num_traits::{Num, SaturatingAdd, SaturatingSub};
use std::{cmp::Ordering, collections::BTreeSet, fmt::Display};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Unit<T: Num + SaturatingAdd> {
    Single(T),
    Range((T, T)), // lower bound and upper bound
}

impl<T: Num + Ord + SaturatingAdd + Copy + Display> Unit<T> {
    fn is_one_less_than(&self, other: &Self) -> bool {
        match self {
            Unit::Single(s) => match other {
                Unit::Single(o) => s < o && s.saturating_add(&T::one()) == *o,
                Unit::Range((ol, _)) => s < ol && s.saturating_add(&T::one()) == *ol,
            },
            Unit::Range((_, sh)) => match other {
                Unit::Single(o) => sh < o && sh.saturating_add(&T::one()) == *o,
                Unit::Range((ol, _)) => sh < ol && sh.saturating_add(&T::one()) == *ol,
            },
        }
    }
    fn is_within(&self, other: &Self) -> bool {
        match self {
            Unit::Single(s) => match other {
                Unit::Single(o) => s == o,
                Unit::Range((ol, oh)) => ol <= s && s <= oh,
            },
            Unit::Range((sl, sh)) => match other {
                Unit::Single(o) => sl <= o && o <= sh,
                Unit::Range((ol, oh)) => sl <= oh && ol <= sh,
            },
        }
    }
    fn right(&self) -> T {
        match self {
            Unit::Single(s) => *s,
            Unit::Range((_, r)) => *r,
        }
    }
    fn left(&self) -> T {
        match self {
            Unit::Single(s) => *s,
            Unit::Range((l, _)) => *l,
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
        self.partial_cmp(other).unwrap_or(Ordering::Greater)
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
        println!("put {value}; we have {}", self);
        if let Some(right) = self.set.last().copied() {
            if value.is_within(&right) {
                println!("{value} is within {right} so do nothing at all");
                return;
            } else if right < value {
                if !right.is_one_less_than(&value) {
                    println!("{value} is more than one bigger than the last entry {right}, so let's insert and do nothing else");
                    self.set.insert(value);
                } else {
                    let l = right.left();
                    println!(
                        "{value} is one less than {right}, so remove {right} and insert {l}-{v} and do nothing else"
                    );
                    self.set.remove(&right);
                    self.set.insert(Unit::Range((l, v)));
                }
                return;
            }
        } else {
            self.set.insert(value);
            return;
        }
        if let Some(left) = self.set.first().copied() {
            if value.is_within(&left) {
                println!("{value} is within {left} so do nothing at all");
                return;
            } else if value < left {
                if !value.is_one_less_than(&left) {
                    println!("{value} is more than one less than the first entry {left}, so let's insert and do nothing else");
                    self.set.insert(value);
                } else {
                    let r = left.right();
                    println!(
                        "{value} is one less than {left}, so remove {value} and insert {v}-{r} and do nothing else"
                    );
                    self.set.remove(&left);
                    self.set.insert(Unit::Range((v, r)));
                }
                return;
            }
        }
        // self.set.insert(value);
        // let p_item = self.set.range(..=value).nth_back(1).copied();
        // let n_item = self.set.range(value..).nth(1).copied();
        let (left, right) = self.left_right(value).unwrap();
        if left.is_one_less_than(&right) || left.is_within(&right) {
            println!("{value} is between {left} and {right}, while those two are adjacent, so let's remove and merge them and do nothing else");
            self.set.remove(&left);
            self.set.remove(&right);
            self.set.insert(Unit::Range((left.left(), right.right())));
            return
        }
        // let items = p_item.and_then(|p| n_item.map(|n| (p.min(n), p.max(n))));
        // if let Some((left, right)) = items {
        if value.is_within(&right) {
            println!("{value} is within {right} so do nothing at all");
        } else if right > value {
            if !value.is_one_less_than(&right) {
                print!("{value} is more than one lesser than the next entry {right}");
                if value > left && left.is_one_less_than(&value) {
                    let l = left.left();
                    println!(" but {value} is one greater than {left}, so let's remove {left} and insert {l}-{v} and do nothing else, ok?");
                    self.set.remove(&left);
                    let insertia = Unit::Range((l, v));
                    if let Some((other_left, other_right)) = self.left_right(insertia) {
                        println!("but we have a left {other_left} and right {insertia}!");
                        if other_left.is_one_less_than(&insertia) || other_left.is_within(&insertia) {
                            let real_left = other_left.min(insertia);
                            let real_right = other_left.max(insertia);
                            let l = real_left.left();
                            let r = real_right.right();
                            let otherinsertia = Unit::Range((l, r));
                            println!("Oh wait we gotta merge {other_left} and {insertia} into {otherinsertia}...!!!!!!!!!!!1111!!!");
                            self.set.remove(&other_left);
                            self.set.insert(otherinsertia);
                        } else {
                            self.set.insert(insertia);
                        }
                    } else if let Some(left) = self.set.last().copied() {
                        if left.is_one_less_than(&insertia)  || left.is_within(&insertia) {
                            let l = left.left();
                            let r = insertia.right();
                            let otherinsertia = Unit::Range((l, r));
                            println!("wait wait waiiaaiiait!!!!! {left} is adjacent to {insertia} so remove {left} and insert {otherinsertia}");
                            self.set.remove(&left);
                            self.set.insert(otherinsertia);
                        } else {
                            self.set.insert(insertia);
                        }
                    } else {
                        self.set.insert(insertia);
                    }
                } else {
                    println!(" and {value} is more than one greater than {left}, so let's insert {value} and do nothing else");
                    self.set.insert(value);
                }
            } else {
                let r = right.right();
                println!("{value} is one less than {right}, so remove {right} and insert {v}-{r} and do nothing else");
                let insertia = Unit::Range((v, r));
                self.set.remove(&right);
                if let Some((other_left, other_right)) = self.left_right(insertia) {
                    println!("but we have a left {other_left} and right {insertia}!");
                    if other_left.is_one_less_than(&insertia) || other_left.is_within(&insertia) {
                        let real_left = other_left.min(insertia);
                        let real_right = other_left.max(insertia);
                        let l = real_left.left();
                        let r = real_right.right();
                        let otherinsertia = Unit::Range((l, r));
                        println!("Oh wait we gotta merge {other_left} and {insertia} into {otherinsertia}...!!!!!!!!!!!1111!!!");
                        self.set.remove(&other_left);
                        self.set.insert(otherinsertia);
                    } else {
                        self.set.insert(insertia);
                    }
                } else if let Some(left) = self.set.last().copied() {
                    if left.is_one_less_than(&insertia) || left.is_within(&insertia) {
                        let l = left.left();
                        let r = insertia.right();
                        let otherinsertia = Unit::Range((l, r));
                        println!("wait wait waiiaaiiait!!!!! {left} is adjacent to {insertia} so remove {left} and insert {otherinsertia}");
                        self.set.remove(&left);
                        self.set.insert(otherinsertia);
                    } else {
                        self.set.insert(insertia);
                    }
                } else {
                    self.set.insert(insertia);
                }
            }
            return
        }

    }

    fn left_right(&mut self, value: Unit<T>) -> Option<(Unit<T>, Unit<T>)> {
        let mut right_items = self.set.split_off(&value);
        right_items.remove(&value);
        let f = right_items.first().and_then(|right| {
            self.set.last().map(|left| (*left, *right))
        });
        self.set.extend(right_items);
        f
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
            29, 36, 22, 1, 0, 37, 30, 8, 24, 16, 2,
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
        assert!(ranger.to_string() == "0-2,4,6-8,11-12,14-25,27-33,35-39");
        drop(ranger);
    }
}

// 0-2,8,4,6-8,11,12,14-25,27-33,35-39
// 0-2,6-8,11-12,14-25,27-33,35-39,4
