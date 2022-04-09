#![feature(iter_intersperse)]

pub mod compare;
pub mod comparisons;
pub mod relations;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::{self, File},
    io::{self, stdout, Read, Seek, SeekFrom},
    ops::Add,
};

use clap::{Parser, Subcommand};
use rusted_gumtree_cvs_git::git::{fetch_repository, read_position, read_position_floating_lines};
use termion::color;

use crate::{
    compare::Comparator,
    comparisons::{ComparedRanges, Comparison, Comparisons},
    relations::{Position, Range, Relations},
};

macro_rules! inv_reset {
    ( Fg ) => {
        color::Bg(color::Reset)
    };
    ( Bg ) => {
        color::Fg(color::Reset)
    };
}
macro_rules! show_code_range {
    ($b:tt{$x:tt ($s:tt) with $cx:tt $px:tt }$a:tt with $c:tt $p:tt ) => {
        print!(
            "{}{}{}{}",
            inv_reset!($p),
            color::$p(color::$c),
            // $b,
            summarize_border(&$b, isize::try_from($s).unwrap() + 5),
            color::$p(color::Reset)
        );
        print!(
            "{}{}{}",
            color::$px(color::$cx),
            summarize_center(&$x, $s),
            color::$px(color::Reset)
        );
        print!(
            "{}{}{}",
            color::$p(color::$c),
            summarize_border(&$a, -isize::try_from($s).unwrap() - 5),
            color::$p(color::Reset)
        );
        println!()
    };
}

fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli);
    match &cli.command {
        Commands::Compare { left, right, .. } => {
            println!("left: {} right:{}", left, right);
            let left_r = handle_file(File::open(left).expect("should be a file")).unwrap();
            let right_r = handle_file(File::open(right).expect("should be a file")).unwrap();

            println!("{:?} {:?}", left_r.len(), right_r.len());

            let mut comp = Comparator::default().compare(left_r, right_r);
            comp.left_name = left.clone();
            comp.right_name = right.clone();
            println!("{}", serde_json::to_string_pretty(&comp).unwrap());
        }
        Commands::Stats { file } => {
            let relations = handle_file(File::open(file).expect("should be a file")).unwrap();
            println!("{:#?}", relations);
            println!("# of relations: {}", relations.len());
            println!(
                "mean # of references: {}",
                relations.iter().map(|x| x.refs.len()).sum::<usize>() / relations.len()
            );
            let mut files = HashSet::<String>::default();
            for x in &relations {
                files.insert(x.decl.file.clone());
                for x in &x.refs {
                    files.insert(x.file.clone());
                }
            }
            println!("# of uniquely mentioned files: {}", files.len());
        }
        Commands::InteractiveDeclarations {
            repository,
            commit,
            baseline,
            evaluated: test,
            ..
        } => {
            let repo = fetch_repository(repository.clone(), "/tmp/hyperastgitresources");
            let bl_rs = handle_file(File::open(baseline).expect("should be a file")).unwrap();
            let t_rs = handle_file(File::open(test).expect("should be a file")).unwrap();
            let comp = Comparator::default().compare(bl_rs, t_rs).into();
            print_comparisons_stats(&comp);
            let per_file = decls_per_file(comp);

            for (f, rs) in per_file {
                let read_position = |p: &Position, z: Option<usize>| {
                    if let Some(z) = z {
                        read_position_floating_lines(&repo, commit, &p.clone().into(), z)
                    } else {
                        read_position(&repo, commit, &p.clone().into())
                            .map(|x| ("".to_string(), x, "".to_string()))
                    }
                };
                // println!("{}:{:?}",f,rs);
                for r in &rs.0 {
                    if rs.1.contains(r) {
                        continue;
                    }
                    let p = r.with(f.clone());
                    let (before, span, after) = read_position(&p, Some(4)).unwrap();
                    println!("baseline {}:", p);

                    show_code_range!(
                        before {
                            span (4) with Magenta Bg
                        } after with LightBlack Fg
                    );
                }
                for r in &rs.1 {
                    if rs.0.contains(r) {
                        // println!("matched {}:", r);
                        let p = r.with(f.clone());
                        let (before, span, after) = read_position(&p, Some(4)).unwrap();
                        println!("exact {}:", p);
                        show_code_range!(
                            before {
                                span (4) with Green Bg
                            } after with LightBlack Fg
                        );
                        continue;
                    }
                    let p = r.with(f.clone());
                    let (before, span, after) = read_position(&p, Some(4)).unwrap_or((
                        p.to_string() + "bugged",
                        p.to_string(),
                        p.to_string(),
                    ));
                    println!("test {}:", p);
                    show_code_range!(
                        before {
                            span (4) with Blue Bg
                        } after with LightBlack Fg
                    );
                }
            }
        }
        Commands::Interactive {
            repository,
            commit,
            baseline,
            evaluated: test,
            only_misses,
            ..
        } => {
            let repo = fetch_repository(repository.clone(), "/tmp/hyperastgitresources");
            let bl_rs = handle_file(File::open(baseline).expect("should be a file")).unwrap();
            let t_rs = handle_file(File::open(test).expect("should be a file")).unwrap();
            let comp = Comparator::default()
                .set_intersection_left(*only_misses)
                .compare(bl_rs, t_rs);
            print_comparisons_stats(&comp);
            for r in &comp.exact {
                let read_position = |p: &Position, z: Option<usize>| {
                    if let Some(z) = z {
                        read_position_floating_lines(&repo, commit, &p.clone().into(), z)
                    } else {
                        read_position(&repo, commit, &p.clone().into())
                            .map(|x| ("".to_string(), x, "".to_string()))
                    }
                    .unwrap()
                };
                if r.left.is_empty() {
                    continue;
                }
                let (before, span, after) = read_position(&r.decl, None);

                println!("decl {}:", r.decl,);

                show_code_range!(
                    before {
                        span (1) with Green Bg
                    } after with LightBlack Fg
                );
                // println!(
                //     "decl {}: \n{}{}{}{}{}{}{}{}{}{}",
                //     r.decl,
                //     color::Bg(color::Reset),
                //     color::Fg(color::LightBlack),
                //     before,
                //     color::Fg(color::Reset),
                //     color::Bg(color::Green),
                //     summarize_center(&span, 1),
                //     color::Bg(color::Reset),
                //     color::Fg(color::LightBlack),
                //     after,
                //     color::Fg(color::Reset),
                // );
                explore_misses(r, &read_position);
            }
        }
    }
    // let repo_name = args
    //     .get(1)
    //     .expect("give an argument like openjdk/jdk or INRIA/spoon"); //"openjdk/jdk";//"INRIA/spoon";
    // let before = args.get(2).map_or("", |x| x);
    // let after = args.get(3).map_or("", |x| x);
    // let dir_path = args.get(4).map_or("", |x| x);
    // let mut out = args.get(5).map_or(Box::new(io::stdout()) as Box<dyn Write>, |x| {
    //     Box::new(File::create(x).unwrap()) as Box<dyn Write>
    // });

    // let p: Vec<Relation> = serde_json::from_str(data);
}

fn decls_per_file(comp: Comparisons) -> BTreeMap<String, (Vec<Range>, Vec<Range>)> {
    let mut per_file: BTreeMap<String, (Vec<Range>, Vec<Range>)> = BTreeMap::default();
    for r in &comp.right {
        let r = &r.decl;
        let aa = &mut per_file.entry(r.file.clone()).or_insert((vec![], vec![])).1;
        let rr = r.clone().into();
        if aa.contains(&rr) {
            continue;
        }
        aa.push(rr)
    }
    for l in &comp.left {
        let l = &l.decl;
        let aa = &mut per_file.entry(l.file.clone()).or_insert((vec![], vec![])).0;

        let ll = l.clone().into();
        if aa.contains(&ll) {
            println!("doubled left {}", l);
            continue;
        }
        aa.push(ll)
    }
    for l in &comp.exact {
        let l = &l.decl;
        let (aa, bb) = &mut per_file.entry(l.file.clone()).or_insert((vec![], vec![]));

        let ll = l.clone().into();
        if !aa.contains(&ll) {
            aa.push(ll);
        }
        let ll = l.clone().into();
        if !bb.contains(&ll) {
            bb.push(ll)
        }
    }
    per_file

    // let mut per_file: BTreeMap<String, (HashSet<Range>, HashSet<Range>)> = BTreeMap::default();
    // for r in &comp.right {
    //     let r = &r.decl;
    //     per_file
    //         .entry(r.file.clone())
    //         .or_insert((Default::default(), Default::default()))
    //         .1
    //         .insert(r.clone().into());
    // }
    // for l in &comp.left {
    //     if comp.left.contains(&l) {
    //         continue;
    //     }
    //     let l = &l.decl;
    //     per_file
    //         .entry(l.file.clone())
    //         .or_insert((Default::default(), Default::default()))
    //         .0
    //         .insert(l.clone().into());
    // }
    // let mut res: BTreeMap<String, (Vec<Range>, Vec<Range>)> = BTreeMap::default();

    // for (k,(l,r)) in per_file {
    //     res.entry(r)
    //     .or_insert(vec![]).e
    // }
    // res
}

fn explore_misses<F: Fn(&Position, Option<usize>) -> (String, String, String)>(
    r: &Comparison,
    read_position: &F,
) {
    println!(
        "{} is correctly referenced {} times : {}",
        r.decl,
        r.exact.len(),
        r.exact
            .iter()
            .take(10)
            .map(|x| x.to_string())
            .collect::<String>()
    );
    println!("but it is missed {} times:", r.left.len());
    for (i, r) in r.per_file.iter().enumerate() {
        println!("({}) {}", i, r);
    }
    let mut current_outside_zoom: Option<usize> = None;
    let mut current_inside_zoom: usize = 1;
    loop {
        println!(
            "do you want to see details ? [NO/Quit/{}]",
            (0..r.per_file.len())
                .into_iter()
                .map(|x| x.to_string())
                .intersperse("/".to_string())
                .collect::<String>()
        );
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        match &buffer[..buffer.len() - 1] {
            "" | "NO" | "n" => break,
            "Quit" | "q" => {
                std::process::exit(0);
            }
            x => {
                if &x[..1] == "*" || &x[..1] == "/" || &x[..1] == "+" || &x[..1] == "-" {
                    let n = if x.len() == 1 {
                        0
                    } else if let Ok(n) = x[1..].parse::<usize>() {
                        n
                    } else {
                        println!("can you repeat ? \"{}\" is invalid", x);
                        continue;
                    };
                    if &x[..1] == "+" {
                        *current_outside_zoom.get_or_insert(0) += 1 + n;
                    } else if &x[..1] == "*" {
                        current_inside_zoom += 1 + n;
                    } else if &x[..1] == "-" {
                        if let Some(z) = current_outside_zoom {
                            if z > 0 {
                                current_outside_zoom = Some(z - n);
                            } else {
                                current_outside_zoom = None;
                            }
                        }
                    } else if &x[..1] == "/" {
                        current_inside_zoom = (current_inside_zoom.saturating_sub(n)).max(1);
                    } else {
                        println!("can you repeat ? \"{}\" is invalid", x);
                        continue;
                    }
                    let (before, span, after) = read_position(&r.decl, current_outside_zoom);
                    println!("decl {}:", r.decl,);
                    show_code_range!(
                        before {
                            span (current_inside_zoom) with Green Bg
                        } after with LightBlack Fg
                    );
                } else if let Ok(n) = x.parse::<usize>() {
                    if n < r.per_file.len() {
                        explore_compared_ranges(&r.per_file[n], read_position)
                    } else {
                        println!("can you repeat ? \"{}\" is not in range", x);
                        // buffer.clear();
                        // io::stdin().read_line(&mut buffer).unwrap();
                    }
                } else {
                    println!("can you repeat ? \"{}\" is invalid", x);
                    buffer.clear();
                    io::stdin().read_line(&mut buffer).unwrap();
                }
            }
        }
    }
}

fn explore_compared_ranges<F>(ranges: &ComparedRanges, read_position: &F)
where
    F: Fn(&Position, Option<usize>) -> (String, String, String),
{
    // let mut screen = AlternateScreen::from(stdout());
    // macro_rules! show {
    //     ( $($arg:tt)* ) => {
    //         write!(screen, $($arg)*).unwrap();
    //         // screen.flush().unwrap();
    //     };
    // }
    println!("in {}\n", ranges.file);
    println!("matched in baseline {} times\n", ranges.left.len());
    println!("but it found incorrectly {} times", ranges.right.len());
    for (i, x) in ranges.left.iter().enumerate() {
        println!("!({}) {}", i, x);
    }
    for (i, x) in ranges.right.iter().enumerate() {
        println!("?({}) {}", ranges.left.len() + i, x);
    }
    let mut current_outside_zoom: Option<usize> = None;
    let mut current_inside_zoom: usize = 1;
    let mut current_choice: Option<usize> = None;
    loop {
        println!("do you want to see details ? [NO/Quit/Help/!#/?#/+#/-#/*#//#]");
        // screen.flush().unwrap();
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        match &buffer[..buffer.len() - 1] {
            "" | "NO" | "n" => break,
            "Quit" | "q" => {
                std::process::exit(0);
            }
            "Help" | "h" => {
                println!(
                    "Help navigating references:
                NO\tn: default go back up to declarations 
                Quit\tq: quit the whole prompt, you can also CTRL-C
                Help\th: current manual
                !#: select a range of a reference found in the baseline
                ?#: select a range of a reference found in the test result to evaluate
                +#: show more lines around current range
                -#: show less lines around current range
                *#: show more lines in the current range
                /#: show less lines in the current range
                ",
                );
            }
            x => {
                if &x[..1] == "+" {
                    if x.len() == 1 {
                        *current_outside_zoom.get_or_insert(0) += 1;
                    } else if let Ok(n) = x[1..].parse::<usize>() {
                        *current_outside_zoom.get_or_insert(0) += 1 + n;
                    } else {
                        println!("can you repeat ? \"{}\" is invalid", x);
                        continue;
                    }
                } else if &x[..1] == "*" {
                    if x.len() == 1 {
                        current_inside_zoom += 1;
                    } else if let Ok(n) = x[1..].parse::<usize>() {
                        current_inside_zoom += 1 + n;
                    } else {
                        println!("can you repeat ? \"{}\" is invalid", x);
                        continue;
                    }
                } else if &x[..1] == "-" {
                    if x.len() == 1 {
                        if let Some(z) = current_outside_zoom {
                            if z > 0 {
                                current_outside_zoom = Some(z - 1);
                            } else {
                                current_outside_zoom = None;
                            }
                        }
                    } else if let Ok(n) = x[1..].parse::<usize>() {
                        if let Some(z) = current_outside_zoom {
                            if z > 0 {
                                current_outside_zoom = Some(z - 1 - n);
                            } else {
                                current_outside_zoom = None;
                            }
                        }
                    } else {
                        println!("can you repeat ? \"{}\" is invalid", x);
                        continue;
                    }
                } else if &x[..1] == "/" {
                    if x.len() == 1 {
                        current_inside_zoom = (current_inside_zoom - 1).max(1);
                    } else if let Ok(n) = x[1..].parse::<usize>() {
                        current_inside_zoom = (current_inside_zoom.saturating_sub(n)).max(1);
                    } else {
                        println!("can you repeat ? \"{}\" is invalid", x);
                        continue;
                    }
                } else if let Ok(n) = x.parse::<usize>() {
                    current_choice = Some(n);
                } else {
                    println!("can you repeat ? \"{}\" is invalid", x);
                    continue;
                }
                if let Some(n) = current_choice {
                    if n < ranges.left.len() {
                        let p = ranges.left[n].with(ranges.file.clone());
                        let (before, span, after) =
                            read_position(&p.clone().into(), current_outside_zoom);
                        println!("show !({}) {}:", n, p);
                        show_code_range!(
                            before {
                                span (current_inside_zoom) with Magenta Bg
                            } after with LightBlack Fg
                        );
                        // println!(
                        //     "show !({}) {}: \n{}{}{}{}{}{}{}{}{}{}",
                        //     n,
                        //     p,
                        //     color::Bg(color::Reset),
                        //     color::Fg(color::LightBlack),
                        //     before,
                        //     color::Fg(color::Reset),
                        //     color::Bg(color::Magenta),
                        //     summarize_center(&span, current_inside_zoom),
                        //     color::Bg(color::Reset),
                        //     color::Fg(color::LightBlack),
                        //     after,
                        //     color::Fg(color::Reset),
                        // );
                        // println!("{:?}", current_outside_zoom);
                    } else if n < ranges.left.len() + ranges.right.len() {
                        let p = ranges.right[n - ranges.left.len()].with(ranges.file.clone());
                        let (before, span, after) =
                            read_position(&p.clone().into(), current_outside_zoom);
                        println!("show ?({}) {}:", n, p);
                        show_code_range!(
                            before {
                                span (current_inside_zoom) with Blue Bg
                            } after with LightBlack Fg
                        );
                        // println!(
                        //     "show ?({}) {}: \n{}{}{}{}{}{}{}{}{}{}",
                        //     n,
                        //     p,
                        //     color::Bg(color::Reset),
                        //     color::Fg(color::LightBlack),
                        //     before,
                        //     color::Fg(color::Reset),
                        //     color::Bg(color::Blue),
                        //     summarize_center(&span, current_inside_zoom),
                        //     color::Bg(color::Reset),
                        //     color::Fg(color::LightBlack),
                        //     after,
                        //     color::Fg(color::Reset),
                        // );
                        // println!("{:?}", current_outside_zoom);
                    } else {
                        continue;
                        // println!("can you repeat ? \"{}\" is not in range", x);
                        // buffer.clear();
                        // io::stdin().read_line(&mut buffer).unwrap();
                    }
                } else {
                    panic!()
                }
            }
        }
    }
}

/// TODO remove temporary things related to analysing spoon codebase
fn print_comparisons_stats(comp: &Comparisons) {
    println!("# of exact decls matches: {}", comp.exact.len());
    println!("# of remaining decls in baseline: {}", comp.left.len());
    let comp_bl = comp
        .exact
        .iter()
        .map(|x| (x.left.len(), x.exact.len()))
        .filter(|x| x.0 != 0 || x.1 != 0)
        .fold((0, 0, 0f64, 0), |(xl, xe, m, c), (x, e)| {
            (x + xl, e + xe, m + (e as f64 / (x + e) as f64), c + 1)
        });
    let comp_tool = comp
        .exact
        .iter()
        .map(|x| {
            (
                x.right
                    .iter()
                    .filter(|x| !x.file.starts_with("spoon-"))
                    .count(),
                x.exact.len(),
            )
        })
        .filter(|x| x.0 != 0 || x.1 != 0)
        .fold((0, 0, 0f64, 0), |(xl, xe, m, c), (x, e)| {
            (x + xl, e + xe, m + (x as f64 / (x + e) as f64), c + 1)
        });
    println!(
        "# of remaining decls in tool results: {}",
        comp.right
            .iter()
            .filter(|x| !x.decl.file.starts_with("spoon-"))
            .count()
    );
    println!("overall success rate: {}", {
        let (x, e, _, _) = comp_bl;
        e as f64 / (x + e) as f64
    });
    println!("overall overestimation rate: {}", {
        let (x, e, _, _) = comp_tool;
        x as f64 / (x + e) as f64
    });
    println!("mean success rate: {}", {
        let (_, _, r, c) = comp_bl;
        r as f64 / c as f64
    });
    println!(
        "mean overestimation rate: {}",{
            let (_, _, r, c) = comp_tool;
            r as f64 / c as f64
        });
    println!(
        "mean # of exact references: {}",
        comp.exact.iter().map(|x| x.exact.len()).sum::<usize>() as f64 / comp.exact.len() as f64
    );
    println!(
        "mean # of remaining refs in baseline: {}",
        comp.exact.iter().map(|x| x.left.len()).sum::<usize>() as f64 / comp.exact.len() as f64
    );
    println!(
        "mean # of remaining refs in tool results: {}",
        comp.exact
            .iter()
            .map(|x| x
                .right
                .iter()
                .filter(|x| !x.file.starts_with("spoon-"))
                .count())
            .sum::<usize>() as f64
            / comp.exact.len() as f64
    );
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
    println!("# of uniquely mentioned files: {}", files.len());
}

fn summarize_center(text: &str, border_lines: usize) -> String {
    let mut before = 0;
    for _ in 0..border_lines {
        let x = text[before..]
            .find(|x: char| x == '\n')
            .unwrap_or(text.len() - before);
        before = before + x + 1;
        before = before.min(text.len() - 1)
    }
    let mut after = text.len();
    for _ in 0..border_lines {
        let x = text[..after].rfind(|x: char| x == '\n').unwrap_or_default();
        after = x;
    }
    if before >= after {
        text.to_string()
    } else {
        let mut r = text[..before].to_string();
        r += "............ ignored ";
        let ignored = after - before;
        r += &ignored.to_string();
        r += " characters ............";
        r += &text[after..];
        r
    }
}
fn summarize_border(text: &str, border_lines: isize) -> String {
    if border_lines == 0 {
        text.to_string()
    } else if border_lines > 0 {
        let mut before = 1;
        for _ in 0..border_lines {
            let x = text[before - 1..]
                .find(|x: char| x == '\n')
                .unwrap_or(text.len() - before);
            before = before + x + 1;
            before = before.min(text.len() - 1)
        }
        if before + 1 >= text.len() {
            text.to_string()
        } else {
            let mut r = text[..before.saturating_sub(1)].to_string();
            r += ",,,,,,,,,, b ignored ";
            let ignored = text.len() - before;
            r += &ignored.to_string();
            r += " characters ,,,,,,,,,,\n";
            r
        }
    } else {
        let mut after = text.len();
        for _ in 0..-border_lines {
            let x = text[..after].rfind(|x: char| x == '\n').unwrap_or_default();
            after = x;
        }
        if after == 0 {
            text.to_string()
        } else {
            let mut r = "\n,,,,,,,,,, a ignored ".to_string();
            let ignored = after;
            r += &ignored.to_string();
            r += " characters ,,,,,,,,,,";
            r += &text[after..];
            r
        }
    }
}

fn handle_file(mut file: File) -> Result<Relations, serde_json::Error> {
    // let c =  Read::by_ref(&mut left).bytes().count();
    // println!("left: {:?} {:?}", left, c);
    // left.seek(SeekFrom::Start(0)).unwrap();
    let c = file.seek(SeekFrom::End(0)).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    println!("file: {:?} {:?}", file, c);

    if let Ok(x) = serde_json::from_reader::<_, Relations>(&mut file) {
        Ok(x)
    } else {
        file.rewind().unwrap();
        let file = Read::by_ref(&mut file);
        let r = "[".as_bytes().chain(file).chain("]".as_bytes());
        serde_json::from_reader::<_, Relations>(r)
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compare relations
    Compare {
        left: String,
        right: String,
        // #[clap(short, long)]
        // aaa: String,

        // #[clap(short, long, default_value_t = 1)]
        // count: u8
    },

    /// Statistics on relations
    Stats { file: String },

    /// look interactively at missed references to exactly matched declarations
    Interactive {
        /// The git repository that we want to analyse
        /// ie. <domain>/user/project
        /// eg. github.com/INRIA/spoon
        #[clap(short, long)]
        repository: String,

        #[clap(short, long)]
        commit: String,

        #[clap(short, long)]
        only_misses: bool,

        /// File that contains referential relations.
        /// It will be used as a baseline
        baseline: String,

        /// File that contains referential relations.
        /// We want to evalute them.
        evaluated: String,
    },

    /// look interactively at missed declarations
    InteractiveDeclarations {
        /// The git repository that we want to analyse
        /// ie. <domain>/user/project
        /// eg. github.com/INRIA/spoon
        #[clap(short, long)]
        repository: String,

        #[clap(short, long)]
        commit: String,

        /// File that contains referential relations.
        /// It will be used as a baseline
        baseline: String,

        /// File that contains referential relations.
        /// We want to evalute them.
        evaluated: String,
    },
}
