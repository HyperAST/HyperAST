use std::{collections::BTreeMap, mem::take};

use crate::{
    comparisons::{Comparison, Comparisons},
    relations::{Position, Range, Relation, Relations},
};

enum ExactCompareDeclResult {
    Exact(Vec<Position>, Vec<Position>),
    Left(Vec<Position>),
    Right(Vec<Position>),
}

pub struct Comparator {
    pub intersection_left: bool,
}

impl Default for Comparator {
    fn default() -> Self {
        Self {
            intersection_left: Default::default(),
        }
    }
}

impl Comparator {
    pub fn set_intersection_left(self, intersection_left: bool) -> Self {
        Self { intersection_left }
    }

    pub fn compare(&self, left: Relations, right: Relations) -> Comparisons {
        let mut m: BTreeMap<Position, ExactCompareDeclResult> = Default::default();
        for x in left {
            m.insert(x.decl, ExactCompareDeclResult::Left(x.refs));
        }
        for x in right {
            match m.entry(x.decl) {
                std::collections::btree_map::Entry::Occupied(mut y) => {
                    let y = y.get_mut();
                    let left = if let ExactCompareDeclResult::Left(y) = y {
                        take(y)
                    } else {
                        continue;
                    };
                    *y = ExactCompareDeclResult::Exact(left, x.refs);
                }
                std::collections::btree_map::Entry::Vacant(y) => {
                    y.insert(ExactCompareDeclResult::Right(x.refs));
                }
            }
        }

        let mut exact = vec![];
        let mut left = vec![];
        let mut right = vec![];

        for (k, v) in m {
            match v {
                ExactCompareDeclResult::Exact(mut left, mut right) => {
                    left.sort();
                    right.sort();
                    left.dedup();
                    right.dedup();
                    let mut intersection = vec![];
                    let mut per_file: BTreeMap<String, (Vec<Range>, Vec<Range>)> =
                        Default::default();
                    let mut remaining = left;
                    let mut not_matched = vec![];
                    for r in right {
                        if let Some(i) = remaining.iter().position(|x| x == &r) {
                            intersection.push(r);
                            remaining.swap_remove(i);
                        } else {
                            per_file
                                .entry(r.file.clone())
                                .or_insert((vec![], vec![]))
                                .1
                                .push(r.clone().into());
                            not_matched.push(r);
                        }
                    }
                    for l in &remaining {
                        per_file
                            .entry(l.file.clone())
                            .or_insert((vec![], vec![]))
                            .0
                            .push(l.clone().into());
                    }
                    let per_file = if self.intersection_left {
                        per_file
                            .into_iter()
                            .filter(|(_, (l, _))| !l.is_empty())
                            .map(|x| x.into())
                            .collect()
                    } else {
                        per_file.into_iter().map(|x| x.into()).collect()
                    };
                    exact.push(Comparison {
                        decl: k,
                        exact: intersection,
                        per_file,
                        left: remaining,
                        right: not_matched,
                        left_contained: vec![],
                        right_contained: vec![],
                    })
                }
                ExactCompareDeclResult::Left(l) => left.push(Relation { decl: k, refs: l }),
                ExactCompareDeclResult::Right(r) => right.push(Relation { decl: k, refs: r }),
            }
        }

        Comparisons {
            left_name: "".to_string(),
            right_name: "".to_string(),
            exact,
            left,
            right,
        }
    }
}
