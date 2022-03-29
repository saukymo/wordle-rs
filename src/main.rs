use std::cmp::max;
use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;

pub mod decision_tree;
use decision_tree::DecisionTree;


const MAX_TURNS: u8 = 5;

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
struct Restriction {
    required_green: BTreeMap<usize, char>,
    required_yellow: BTreeMap<char, usize>
}

impl Restriction {
    pub fn new() -> Self {
        Restriction {
            required_green: BTreeMap::new(),
            required_yellow: BTreeMap::new()
        }
    }

    pub fn from(guess: &str, pattern: u8) -> Self {
        let mut required_green = BTreeMap::new();
        let mut required_yellow = BTreeMap::new();


        let mut current = pattern;
        for (i, c) in guess.chars().enumerate() {
            let p = current % 3;
            match p {
                2 => { required_green.insert(i, c); },
                1 => { *required_yellow.entry(c).or_insert(0) += 1 },
                _ => ()
            }

            current /= 3;
            if current == 0 {
                break
            }
        }

        Restriction {
            required_green,
            required_yellow
        }
    }

    pub fn merge(&self, other: &Restriction) -> Self{
        let mut restriction = self.clone();

        for (pos, c) in other.required_green.iter() {
            restriction.required_green.insert(*pos, *c);
        }

        for (c, other_count) in other.required_yellow.iter() {
            let self_count = restriction.required_yellow.entry(*c).or_insert(0);
            *self_count = max(*self_count, *other_count)
        }

        restriction
    }

    pub fn evaluate(&self, guess: &str) -> bool {
        for (pos, required_char) in self.required_green.iter() {
            if let Some(c)  = guess.chars().nth(*pos) {
                if c != *required_char {
                    return false;
                }
            }
        }   

        let mut counter: BTreeMap<char, usize> = BTreeMap::new();
        for c in guess.chars() {
            *counter.entry(c).or_insert(0) += 1
        }

        for (c, required_count) in self.required_yellow.iter() {
            if counter.get(&c).unwrap_or(&0) < &required_count {
                return false;
            }
        }

        true
    }
}

struct Checker {
}

impl Checker {
    pub fn check(target: &str, guess: &str) -> u8 {
        let mut freq = BTreeMap::<char, usize>::new();
        for (guess_c, target_c) in guess.chars().zip(target.chars()) {
            if guess_c != target_c {
                let counter = freq.entry(target_c).or_insert(0);
                *counter += 1;
            }
        }

        let mut pattern: u8 = 0;
        let mut base: u8 = 1;
        for (guess_c, target_c) in guess.chars().zip(target.chars()) {
            if guess_c == target_c {
                pattern += 2 * base;
            } else {
                let counter = freq.entry(guess_c).or_insert(0);
                if *counter > 0 {
                    pattern += base;
                    *counter -= 1;
                } 
            }

            base *= 3;
        }

        pattern
    }

    pub fn is_success_pattern(pattern: u8) -> bool {
        pattern == 242
    }
}

struct Solver<'a> {
    // decision_tree: DecisionTree<'a>,
    current: &'a DecisionTree<'a>
}

impl<'a> Solver<'a> {
    pub fn guess(&self) -> String {
        self.current.guess()
    }
}

struct Evaluator<'a> {
    answers: &'a BTreeSet<&'a str>,
    words: &'a BTreeSet<&'a str>
}

impl<'a> Evaluator<'_> {
    pub fn evaluate(&self, decision_tree: DecisionTree, is_hard:bool) {
        let mut total = 0;
        let mut max_turn = 0;
        for answer in self.answers.iter() {

            println!("============{}============", answer);

            let mut solver = Solver {
                current: &decision_tree
            };
            
            let mut allowed = self.words.clone();
            let mut restrictions = Restriction::new();

            let mut turns = 0;
            loop {
                let guess = solver.guess();
                
                assert_eq!(allowed.contains(guess.as_str()), true);

                let pattern = Checker::check(answer, &guess);

                println!("{}: {} {}", turns, guess, pattern);

                turns += 1;
                if Checker::is_success_pattern(pattern) {
                    break;
                }

                solver.current = DecisionTree::next(solver.current, pattern);

                if is_hard {
                    restrictions = restrictions.merge(&Restriction::from(&guess, pattern));
                    allowed = filter_available_guesses(&restrictions, &allowed);
                }

                assert!(turns < 10, "No answer less than 10.");
            }

            total += turns;
            max_turn = max(max_turn, turns)
        }

        println!("Total: {}, Avg: {}, Max: {}", total, total as f32 / self.answers.len() as f32, max_turn);
    }
}


fn group_by_pattern<'a>(guess: &'a str, answers: &BTreeSet<&'a str>) -> BTreeMap<u8, BTreeSet<&'a str>>{
    let mut groups = BTreeMap::new();
    for answer in answers.into_iter() {
        let pattern = Checker::check(answer, guess);
        (*groups.entry(pattern).or_insert_with(BTreeSet::new)).insert(answer.clone());
    }
    groups
}


fn get_entropy(pattern: u8, length: u32) -> u32 {

    if pattern == 242 {
        return 0
    } else {
        return 2 * length - 1;
    }
    
}

fn get_lower_bound_level(length: usize) -> u8 {

    match length {
        1 => 1,
        2..=243 => 2,
        _ => 3
    }
}

fn get_entropy_sum<'a>(guess: &'a str, answers: &BTreeSet<&'a str>) -> (&'a str, u32, BTreeMap<u8, BTreeSet<&'a str>>) {
    let groups = group_by_pattern(guess, answers);

    let entropy = groups.iter().map(|(pattern, group)| {
        get_entropy(*pattern, group.len() as u32)
    }).sum();

    // println!("For guess {}, # of answers: {}, # of valid groups: {}", guess, answers.len(), groups.len());
    // println!("{:?}", groups);

    (guess, entropy, groups)
}

fn filter_available_guesses<'a> (restriction: &Restriction, words: &BTreeSet<&'a str>) -> BTreeSet<&'a str> {
    words.iter().filter(|word| {
        restriction.evaluate(word)
    }).cloned().collect()
}

#[derive(Debug, Clone, PartialEq)]
struct Best<'a> {
    has_result: bool,
    max_level: u8,
    total_count: u32,
    decision_tree: DecisionTree<'a>
}

impl<'a> Best<'a> {
    pub fn new() -> Self {
        Best {
            has_result: false,
            max_level: 0,
            total_count: u32::MAX,
            decision_tree: DecisionTree::new()
        }
    }

    pub fn init(guess: &'a str, total_count: u32) -> Self {
        Best {
            has_result: true,
            max_level: 0,
            total_count: total_count,
            decision_tree: DecisionTree::from(guess, BTreeMap::new())
        }
    }

    pub fn better(&mut self, other: Best<'a>) {
        if !other.has_result {
            return;
        }

        if self.total_count > other.total_count || (self.total_count == other.total_count && self.max_level > other.max_level + 1) {
            self.max_level = other.max_level + 1;
            self.total_count = other.total_count;
            self.decision_tree = other.decision_tree;
            self.has_result = true;
        }
    }

    pub fn update(&mut self, pattern: u8, other: Best<'a>) {
        if !other.has_result {
            return;
        }

        self.max_level = max(self.max_level, other.max_level);
        self.total_count += other.total_count;
        self.decision_tree.add_branch(pattern, other.decision_tree);
        self.has_result = true;
    }
}   


fn dfs_starter<'a>(start_word: &'a str, answers: &BTreeSet<&'a str>, availables: &BTreeSet<&'a str>) -> Best<'a> {
    let groups = group_by_pattern(start_word, answers);
    let mut current_guess = Best::init(start_word, answers.len() as u32);

    let mut sorted_groups: Vec<_> = groups.into_iter().collect();
    sorted_groups.sort_unstable_by_key(|(_, g)| g.len()); 


   let bests: Vec<_> = sorted_groups.par_iter().map(|(pattern, pattern_answers)| {
        let best = if Checker::is_success_pattern(*pattern) {
            Best {
                has_result: true,
                max_level: 0,
                total_count: 0,
                decision_tree: DecisionTree::new()
            }
        } else if pattern_answers.len() == 1{
            Best {
                has_result: true,
                max_level: 1,
                total_count: 1,
                decision_tree: DecisionTree::from(pattern_answers.iter().nth(0).unwrap(), BTreeMap::from([(242, DecisionTree::new())]))
            }
        } else if pattern_answers.len() <= 3 {
            dfs(1, &pattern_answers, &pattern_answers)
        } else {
            let new_restrictions = Restriction::from(start_word, *pattern);
            dfs(1, &pattern_answers, &filter_available_guesses(&new_restrictions, &availables))
        };
        (pattern, best)
    }).collect();

    for (pattern, best) in bests {
        current_guess.update(*pattern, best);
    }

    current_guess.max_level += 1;
    current_guess
}

fn dfs<'a>(current: u8, answers: &BTreeSet<&'a str>, availables: &BTreeSet<&'a str>) -> Best<'a> {

    if current > MAX_TURNS {
        return Best::new();
    }

    let mut best_of_all_guess = Best::new();

    let mut group_patterns = BTreeSet::<BTreeMap<u8, BTreeSet<&str>>>::new();
    let mut preprocess_by_guess:Vec<_> = availables
        .iter()
        .filter_map(|guess| {
            let (guess, entropy, groups) = get_entropy_sum(guess, &answers);
            if group_patterns.contains(&groups) {
                return None
            }
    
            group_patterns.insert(groups.clone());

            return Some((guess, entropy, groups))
        })
        .collect();
    
    preprocess_by_guess.sort_by_cached_key(|(_, entropy, _)| *entropy);

    for (guess, entropy, groups) in preprocess_by_guess {

        let mut lower_bound = entropy;
        let mut current_guess = Best::init(guess, answers.len() as u32);

        if current_guess.total_count + lower_bound > best_of_all_guess.total_count {
            continue;
        }

        let mut sorted_groups: Vec<_> = groups.into_iter().collect();
        sorted_groups.sort_unstable_by_key(|(_, g)| g.len()); 

        for (pattern, pattern_answers) in sorted_groups {

            let sub_result = if Checker::is_success_pattern(pattern) {
                Best {
                    has_result: true,
                    max_level: 0,
                    total_count: 0,
                    decision_tree: DecisionTree::new()
                }
            } else if pattern_answers.len() == 1{
                Best {
                    has_result: true,
                    max_level: 1,
                    total_count: 1,
                    decision_tree: DecisionTree::from(pattern_answers.iter().nth(0).unwrap(), BTreeMap::from([(242, DecisionTree::new())]))
                }
            } else if pattern_answers.len() <= 3 {
                dfs(current + 1, &pattern_answers, &pattern_answers)
            } else {
                let new_restrictions = Restriction::from(guess, pattern);
                dfs(current + 1, &pattern_answers, &filter_available_guesses(&new_restrictions, &availables))
            };
            
            if !sub_result.has_result {
                current_guess.has_result = false;
                break
            }

            current_guess.update(pattern, sub_result);

            let current_entropy = get_entropy(pattern, pattern_answers.len() as u32);
            lower_bound -= current_entropy;

            if current_guess.total_count  + lower_bound > best_of_all_guess.total_count {
                current_guess.has_result = false;
                break
            }
        }

        if current_guess.has_result {
            best_of_all_guess.better(current_guess);
        }
    }

    best_of_all_guess
}

fn main() {
    // 1075, total 3587, max 6, time: 15.02s
    let answers: BTreeSet<_> = include_str!("../data/answers.txt").lines().take(1000).collect();
    let words: BTreeSet<_> = include_str!("../data/words.txt").lines().collect();

    let best = dfs_starter("salet", &answers, &words);

    println!("{}, {}", best.max_level, best.total_count);
    
    best.decision_tree.to_json().unwrap();

    let evaluator = Evaluator {
        answers: &answers,
        words: &words
    };

    evaluator.evaluate(best.decision_tree, true);
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_check() {
        assert_eq!(Checker::check("admin", "crash"), 9);
        assert_eq!(Checker::check("abbbb", "caccc"), 3);
        assert_eq!(Checker::check("babbb", "caccc"), 6);
        assert_eq!(Checker::check("aabbb", "ccaac"), 36);
        assert_eq!(Checker::check("aabbb", "cccac"), 27);
        assert_eq!(Checker::check("aabbb", "caccc"), 6);
        assert_eq!(Checker::check("baabb", "acaac"), 19);
        assert_eq!(Checker::check("aaaar", "error"), 162); // BBBBG
    }

    #[test]
    fn test_if_success() {
        assert_eq!(Checker::is_success_pattern(9), false);
        assert_eq!(Checker::is_success_pattern(242), true);
    }

    #[test]
    fn test_group_by_pattern() {
        assert_eq!(group_by_pattern("salet", &BTreeSet::from(["sblet", "sclet", "zzzzz"])), BTreeMap::from([
           (236, BTreeSet::from(["sblet", "sclet"])), // GBGGG
           (0, BTreeSet::from(["zzzzz"])) 
        ]));
    }

    #[test]
    fn test_restriction() {
        let mut restriction_a= Restriction {
            required_green: BTreeMap::from([(1, 'a')]),
            required_yellow: BTreeMap::from([('c', 2)]),
        };
        
        let restriction_b = Restriction::from("azczz", 11);
        assert_eq!(restriction_b, Restriction{
            required_green: BTreeMap::from([(0, 'a')]),
            required_yellow: BTreeMap::from([('c', 1)]),
        });

        restriction_a = restriction_a.merge(&restriction_b);
        assert_eq!(restriction_a, Restriction{
            required_green: BTreeMap::from([(0, 'a'), (1, 'a')]),
            required_yellow: BTreeMap::from([('c', 2)]),
        });

        assert_eq!(restriction_a.evaluate("aazcc"), true);
        assert_eq!(restriction_a.evaluate("aaccz"), true);
        assert_eq!(restriction_a.evaluate("azbcc"), false);
        assert_eq!(restriction_a.evaluate("aabbc"), false);
    }

    #[test]
    fn test_filter_guesses() {
        let restriction = Restriction {
            required_green: BTreeMap::from([(0, 'a'), (1, 'a')]),
            required_yellow: BTreeMap::from([('c', 2)]),
        };

        let words = BTreeSet::from(["aazcc", "aaccz", "azbcc", "aabbc"]);

        assert_eq!(filter_available_guesses(&restriction, &words), BTreeSet::from(["aazcc", "aaccz"]));
    }

    #[test]
    fn test_decision_tree() {
        let a = DecisionTree::from("fiveb", BTreeMap::from([]));
        let b = DecisionTree::from("salte", BTreeMap::from([]));

        let mut c = &DecisionTree::from("salet", BTreeMap::from([
            (0, a),
            (134, b),
        ]));
        
        assert_eq!(c.guess(), "salet");
        c = DecisionTree::next(&c, 134);
        assert_eq!(c.guess(), "salte");
        
    }

    #[test]
    fn test_single_search() {
        let best = dfs(0, &BTreeSet::from(["salet"]), &BTreeSet::from(["salet"]));
        assert_eq!(best, Best {
            has_result: true,
            max_level: 1,
            total_count: 1,
            decision_tree: DecisionTree::from("salet", BTreeMap::from([
                (242, DecisionTree::new())
            ]))
        })
    }

    #[test]
    fn test_a_few_search() {
        let answers = BTreeSet::from(["aback", 
        "abase",
        "abate",
        "abbey",
        "abbot",
        "abhor",
        "abide",
        "abled",
        "abode",
        "abort"]);
    
        let words = BTreeSet::from(["aback", 
        "abase",
        "abate",
        "abbey",
        "abbot",
        "abhor",
        "abide",
        "abled",
        "abode",
        "abort",
        "salet"]);

        let best = dfs(0, &answers, &words);
        assert_eq!(best.has_result, true);
        assert_eq!(best.max_level, 3);
        assert_eq!(best.total_count, 21); 

        let evaluator = Evaluator {
            answers: &answers,
            words: &words
        };

        evaluator.evaluate(best.decision_tree, true);

    }

    #[test]
    fn test_lower_bound_level() {
        assert_eq!(get_lower_bound_level(1), 1);
        assert_eq!(get_lower_bound_level(2), 2);
        assert_eq!(get_lower_bound_level(5), 2);
        assert_eq!(get_lower_bound_level(243), 2);
        assert_eq!(get_lower_bound_level(244), 3);
    }

    #[test]
    fn test_get_entropy() {
        assert_eq!(get_entropy(242, 1), 0);
        assert_eq!(get_entropy(0, 2), 3);
        assert_eq!(get_entropy(0, 5), 9);
        assert_eq!(get_entropy(0, 244), 487);
    }

    #[test]
    fn test_a_few_search_with_starter() {
        let answers = BTreeSet::from(["aback", 
        "abase",
        "abate",
        "abbey",
        "abbot",
        "abhor",
        "abide",
        "abled",
        "abode",
        "abort"]);
    
        let words = BTreeSet::from(["aback", 
        "abase",
        "abate",
        "abbey",
        "abbot",
        "abhor",
        "abide",
        "abled",
        "abode",
        "abort",
        "salet"]);

        let best = dfs_starter("salet", &answers, &words);
        assert_eq!(best.has_result, true);
        assert_eq!(best.max_level, 3);
        assert_eq!(best.total_count, 23); 

        let evaluator = Evaluator {
            answers: &answers,
            words: &words
        };

        evaluator.evaluate(best.decision_tree, true);
    }
}