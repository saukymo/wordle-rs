use std::sync::{Arc, Mutex};
use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;

use crate::game::{Checker};
use crate::common::{Best, DecisionTree, Restriction, Task, Cache, Counter};
use crate::utils::*;
use crate::dfs::{dfs, dfs_with_cache};    

pub fn parallel_wrapper<'a>(start_word: &'a str, answers: &BTreeSet<&'a str>, availables: &BTreeSet<&'a str>) -> Best<'a> {
    let mut tasks :BTreeSet<Task<'a>> = BTreeSet::new();

    let groups = group_by_pattern(start_word, answers);
    let mut sorted_groups: Vec<_> = groups.into_iter().collect();
    sorted_groups.sort_unstable_by_key(|(_, g)| g.len()); 

    let mut start_best = Best::init(start_word, answers.len() as u32);

    let mut answers_count: BTreeMap<u8, usize> = BTreeMap::new();

    for (pattern, pattern_answers) in sorted_groups {

        if Checker::is_success_pattern(pattern) {
            start_best.update(pattern, Best{
                has_result: true,
                max_level: 0,
                total_count: 0,
                decision_tree: DecisionTree::new()
            });
            continue;
        }

        if pattern_answers.len() == 1{
            start_best.update(pattern, Best {
                has_result: true,
                max_level: 1,
                total_count: 1,
                decision_tree: DecisionTree::from(pattern_answers.iter().nth(0).unwrap(), BTreeMap::from([(242, DecisionTree::new())]))
            });
            continue;
        };   

        answers_count.insert(pattern, pattern_answers.len());

        let restriction = Restriction::from(start_word, pattern);
        let available_guesses = filter_available_guesses(&restriction, &availables);

        let mut group_patterns = BTreeSet::<BTreeMap<u8, BTreeSet<&str>>>::new();
        for second_guess in available_guesses.iter() {
            let second_groups = group_by_pattern(second_guess, &pattern_answers);

            if group_patterns.contains(&second_groups) {
                continue
            }
            group_patterns.insert(second_groups.clone());

            for (second_pattern, _) in second_groups {
                tasks.insert((pattern, second_guess, second_pattern));
            }
        }

    }

    println!("Prepared Tasks.");

    let bests: Vec<_> = tasks.par_iter().map(|(pattern, second_guess, second_pattern)|{
        if Checker::is_success_pattern(*second_pattern) {
            return (pattern, second_guess, second_pattern, Best{
                has_result: true,
                max_level: 0,
                total_count: 0,
                decision_tree: DecisionTree::new()
            });
        }

        let first_restriction = Restriction::from(start_word, *pattern);
        let second_restriction = Restriction::from(second_guess, *second_pattern);
        
        let available_guesses = filter_available_guesses(&first_restriction, availables);
        let available_guesses = filter_available_guesses(&second_restriction, &available_guesses);

        if available_guesses.len() == 1 {
            return (pattern, second_guess, second_pattern, Best{
                has_result: true,
                max_level: 1,
                total_count: 1,
                decision_tree: DecisionTree::from(available_guesses.iter().nth(0).unwrap(), BTreeMap::from([(242, DecisionTree::new())]))
            })
        }

        let available_answers = filter_available_answers(start_word, *pattern, answers);
        let available_answers = filter_available_answers(second_guess, *second_pattern, &available_answers);

        (pattern, second_guess, second_pattern, dfs(2, &available_answers, &available_guesses))
    }).collect();

    let mut results: BTreeMap<u8, BTreeMap<&str, BTreeMap<u8, Best>>> = BTreeMap::new();

    println!("Finished Tasks.");

    for (pattern, second_guess, second_pattern, best) in bests {
        results
            .entry(*pattern)
            .or_insert_with(BTreeMap::new)
            .entry(second_guess)
            .or_insert_with(BTreeMap::new)
            .insert(*second_pattern, best);
    }

    for (pattern, pattern_results) in results {
        let mut best_of_all_guess = Best::new();

        for (second_guess, second_guess_result) in pattern_results {
            let answer_count = answers_count.get(&pattern).unwrap();

            let mut current_guess = Best::init(second_guess, *answer_count as u32);

            for (second_pattern, best) in second_guess_result {
                if best.has_result {
                    current_guess.update(second_pattern, best);
                } else {
                    current_guess.has_result = false;
                    break
                }
            }

            if current_guess.has_result {
                best_of_all_guess.better(current_guess);
            }
        }

        assert_eq!(best_of_all_guess.has_result, true);
        start_best.update(pattern, best_of_all_guess);
    }

    println!("Found Best.");

    start_best.max_level += 1;
    return start_best
}

pub fn start_word_wrapper<'a>(start_word: &'a str, answers: &BTreeSet<&'a str>, availables: &BTreeSet<&'a str>) -> Best<'a> {
    let groups = group_by_pattern(start_word, answers);
    let mut current_guess = Best::init(start_word, answers.len() as u32);

    let mut sorted_groups: Vec<_> = groups.into_iter().collect();
    sorted_groups.sort_unstable_by_key(|(_, g)| g.len()); 

    let bests: Vec<_> = sorted_groups.iter().map(|(pattern, pattern_answers)| {
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

pub fn baseline_wrapper<'a>(start_word: &'a str, answers: &BTreeSet<&'a str>, availables: &BTreeSet<&'a str>) -> Best<'a> {
    let groups = group_by_pattern(start_word, answers);
    let mut current_guess = Best::init(start_word, answers.len() as u32);

    let mut sorted_groups: Vec<_> = groups.into_iter().collect();
    sorted_groups.sort_unstable_by_key(|(_, g)| g.len()); 

    let cache = Arc::new(Mutex::new(Cache::new()));

    let mut counter = Counter {
        result_counter: 0,
        no_result_counter: 0,
        baseline_counter: 0
    };

    sorted_groups.iter().for_each(|(pattern, pattern_answers)| {
        if Checker::is_success_pattern(*pattern) {
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
            let new_restrictions = Restriction::from(start_word, *pattern);
            dfs_with_cache(1, &pattern_answers, &pattern_answers, new_restrictions, &cache, true, &mut counter)
        } else {
            let new_restrictions = Restriction::from(start_word, *pattern);
            dfs_with_cache(1, &pattern_answers, &filter_available_guesses(&new_restrictions, &availables), new_restrictions, &cache, true, &mut counter)
        };
    });

    /*
        First Stage Finished.
            Counter: Counter { result_counter: 432, no_result_counter: 0, baseline_counter: 33 }
        Second Stage Finished.
            Counter: Counter { result_counter: 18247, no_result_counter: 0, baseline_counter: 130 }

        Second Stage Finished.
            Counter: Counter { result_counter: 17627, no_result_counter: 0, baseline_counter: 170 }
    */
    println!("First Stage Finished.");
    println!("Counter: {:?}", counter);

    let mut counter = Counter {
        result_counter: 0,
        no_result_counter: 0,
        baseline_counter: 0
    };

    let bests: Vec<_> = sorted_groups.iter().map(|(pattern, pattern_answers)| {
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
            let new_restrictions = Restriction::from(start_word, *pattern);
            dfs_with_cache(1, &pattern_answers, &pattern_answers, new_restrictions, &cache, false, &mut counter)
        } else {
            let new_restrictions = Restriction::from(start_word, *pattern);
            dfs_with_cache(1, &pattern_answers, &filter_available_guesses(&new_restrictions, &availables), new_restrictions, &cache, false, &mut counter)
        };
        (pattern, best)
    }).collect();

    println!("Second Stage Finished.");
    println!("Counter: {:?}", counter);

    for (pattern, best) in bests {
        current_guess.update(*pattern, best);
    }

    current_guess.max_level += 1;
    current_guess
}