
use std::{collections::HashSet, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::comparisons::{Comparisons, Comparison};

#[derive(Serialize, Deserialize, Debug)]
pub struct CompStats {
    exact_decls_matches: usize,
    decls_in_baseline_and_tool_with_refs: usize,
    remaining_decls_in_baseline: usize,
    remaining_decls_in_tool_results: usize,
    overall_success_rate: f64,
    overall_overestimation_rate: f64,
    mean_success_rate: f64,
    mean_overestimation_rate: f64,
    mean_of_exact_references: f64,
    mean_of_remaining_refs_in_baseline: f64,
    mean_of_remaining_refs_in_tool_results: f64,
    files_len: usize,
}

struct ConfuTable {
    t_positives: usize,
    f_positives: usize,
    f_negatives: usize,
}

struct StatsAccu {
    t_positives: usize,
    f_positives: usize,
    f_negatives: usize,
    total_precision: f64,
    total_recall: f64,
    count: usize,
}

impl CompStats {
    pub fn compute(comp: &Comparisons) -> Self {

        let exact_decls_matches = comp.exact.len();
        let remaining_decls_in_baseline = comp.left.len();
        // let comp_bl = comp
        let accu = comp
            .exact
            .iter()
            .map(Into::<ConfuTable>::into)
            .filter(ConfuTable::is_not_zero)
            .fold(StatsAccu::default(), StatsAccu::acc);
        // .fold((0, 0, 0f64, 0), |(xl, xe, m, c), (x, e)| {
        //     (x + xl, e + xe, m + (e as f64 / (x + e) as f64), c + 1)
        // });
        // let comp_tool = comp
        //     .exact
        //     .iter()
        //     .map(|x| (x.right.iter().count(), x.exact.len()))
        //     .filter(|x| x.0 != 0 || x.1 != 0)
        //     .fold((0, 0, 0f64, 0), |(xl, xe, m, c), (x, e)| {
        //         (x + xl, e + xe, m + (x as f64 / (x + e) as f64), c + 1)
        //     });
        let remaining_decls_in_tool_results = comp.right.iter().count();
        let overall_success_rate = {
            accu.t_positives as f64 / (accu.f_negatives + accu.t_positives) as f64
            // let (x, e, _, _) = comp_bl;
            // e as f64 / (x + e) as f64
        };
        let overall_overestimation_rate = {
            accu.t_positives as f64 / (accu.f_positives + accu.t_positives) as f64
            // let (x, e, _, _) = comp_tool;
            // x as f64 / (x + e) as f64
        };
        let mean_success_rate = {
            accu.total_recall as f64 / accu.count as f64
            // let (_, _, r, c) = accu;
            // r as f64 / c as f64
        };

        let mean_overestimation_rate = {
            accu.total_precision as f64 / accu.count as f64
            // let (_, _, r, c) = comp_tool;
            // r as f64 / c as f64
        };

        let mean_of_exact_references = accu.t_positives as f64 / (accu.count as f64 + 0.000001);
        // comp.exact.iter().map(|x| x.exact.len()).sum::<usize>()
        //     as f64
        //     / comp.exact.len() as f64;

        let mean_of_remaining_refs_in_baseline = accu.f_negatives as f64 / (accu.count as f64 + 0.000001);
            // comp.exact.iter().map(|x| x.left.len()).sum::<usize>() as f64 / comp.exact.len() as f64;

        let mean_of_remaining_refs_in_tool_results = accu.f_positives as f64 / (accu.count as f64 + 0.000001);
        // comp
        //     .exact
        //     .iter()
        //     .map(|x| x.right.iter().count())
        //     .sum::<usize>() as f64
        //     / comp.exact.len() as f64;
        let mut files = HashSet::<String>::default();
        for x in &comp.exact {
            files.insert(x.decl.file.clone());
            for x in &x.exact {
                files.insert(x.file.clone());
            }
            for x in &x.left {
                files.insert(x.file.clone());
            }
            for x in &x.right {
                files.insert(x.file.clone());
            }
        }
        for x in &comp.left {
            files.insert(x.decl.file.clone());
            for x in &x.refs {
                files.insert(x.file.clone());
            }
        }
        for x in &comp.right {
            files.insert(x.decl.file.clone());
            for x in &x.refs {
                files.insert(x.file.clone());
            }
        }
        let files_len = files.len();
        let decls_in_baseline_and_tool_with_refs = 
        comp
            .exact
            .iter()
            .map(|x| (x.left.len(), x.exact.len()))
            .count();
        Self {
            exact_decls_matches,
            remaining_decls_in_baseline,
            remaining_decls_in_tool_results,
            decls_in_baseline_and_tool_with_refs,
            overall_success_rate,
            overall_overestimation_rate,
            mean_success_rate,
            mean_overestimation_rate,
            mean_of_exact_references,
            mean_of_remaining_refs_in_baseline,
            mean_of_remaining_refs_in_tool_results,
            files_len,
        }
    }
}

impl Display for CompStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("# of exact decls matches: {}", self.exact_decls_matches);
        println!(
            "# of remaining decls in baseline: {}",
            self.remaining_decls_in_baseline
        );
        println!(
            "# of remaining decls in tool results: {}",
            self.remaining_decls_in_tool_results
        );
        println!(
            "# of decls with refs in baseline or tool: {}",
            self.decls_in_baseline_and_tool_with_refs
        );
        println!("overall success rate: {}", self.overall_success_rate);
        println!(
            "overall overestimation rate: {}",
            self.overall_overestimation_rate
        );
        println!("mean success rate: {}", self.mean_success_rate);
        println!(
            "mean overestimation rate: {}",
            self.mean_overestimation_rate
        );
        println!(
            "mean # of exact references: {}",
            self.mean_of_exact_references
        );
        println!(
            "mean # of remaining refs in baseline: {}",
            self.mean_of_remaining_refs_in_baseline
        );
        println!(
            "mean # of remaining refs in tool results: {}",
            self.mean_of_remaining_refs_in_tool_results
        );
        writeln!(f, "# of uniquely mentioned files: {}", self.files_len)
    }
}

impl ConfuTable {
    fn precision(&self) -> f64 {
        let positives = (self.t_positives + self.f_positives) as f64;
        self.t_positives as f64 / positives
    }
    fn recall(&self) -> f64 {
        let accurates = (self.t_positives + self.f_negatives) as f64;
        self.t_positives as f64 / accurates
    }
    fn is_not_zero(&self) -> bool {
        self.t_positives != 0 || self.f_positives != 0 || self.f_negatives != 0
    }
}

impl<'a> From<&'a Comparison> for ConfuTable {
    fn from(x: &'a Comparison) -> Self {
        Self {
            t_positives: x.exact.len(),
            f_positives: x.right.len(),
            f_negatives: x.left.len(),
        }
    }
}

impl Default for StatsAccu {
    fn default() -> Self {
        Self {
            t_positives: 0,
            f_positives: 0,
            f_negatives: 0,
            total_precision: 0.,
            total_recall: 0.,
            count: 0,
        }
    }
}

impl StatsAccu {
    fn acc(self, x: ConfuTable) -> Self {
        Self {
            t_positives: self.t_positives + x.t_positives,
            f_positives: self.f_positives + x.f_positives,
            f_negatives: self.f_negatives + x.f_negatives,
            total_precision: self.total_precision+x.precision(),
            total_recall: self.total_recall+x.recall(),
            count: self.count+1,
        }
    }
}