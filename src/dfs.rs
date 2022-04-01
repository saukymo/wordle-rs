use std::sync::{Arc, Mutex};
use std::collections::{BTreeMap, BTreeSet};

use crate::utils::*;
use crate::common::{Restriction, Best, Cache, DecisionTree, Counter};
use crate::game::Checker;

const MAX_TURNS: u8 = 5;


pub fn dfs_with_cache<'a>(current: u8, answers: &BTreeSet<&'a str>, availables: &BTreeSet<&'a str>, restrictions: Restriction, cache:&Arc<Mutex<Cache<'a>>>, use_limit: bool, counter:&mut Counter) -> Best<'a> {

    if current > MAX_TURNS {
        return Best::new();
    }

    let mut best_of_all_guess = Best::new();

    if let Some(restrictions_cache) = cache.lock().unwrap().get(&restrictions) {
        if let Some(answers_cache) = restrictions_cache.get(answers) {

            // Cached Result:
            if let Some(level_cache) = answers_cache.get(&current) {
                counter.result_counter += 1;
                return level_cache.clone();
            }

            // Cached No Result:
            // for level in 0..(current - 1) {
            //     if let Some(level_cache) = answers_cache.get(&level) {
            //         if !level_cache.has_result || level_cache.max_level + current <= MAX_TURNS {
            //             counter.no_result_counter += 1;
            //             return level_cache.clone()
            //         }
            //     }
            // }
            
            // Cached Base Line:
            for level in (current + 1) .. MAX_TURNS {
                if let Some(level_cache) = answers_cache.get(&level) {
                    counter.baseline_counter += 1;
                    best_of_all_guess = level_cache.clone();
                    break
                }
            }
        }
    }

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
    
    preprocess_by_guess
        .sort_by_cached_key(|(_, entropy, _)| *entropy);
    
    let top_guesses:Vec<_> = if use_limit {
        let length = preprocess_by_guess.len();
        preprocess_by_guess
            .into_iter()
            .take(limit(length))
            .collect()
    } else {
        preprocess_by_guess
    };
    
    for (guess, entropy, groups) in top_guesses {

        let mut lower_bound = entropy;
        let mut current_guess = Best::init(guess, answers.len() as u32);

        if best_of_all_guess.has_result && current_guess.total_count + lower_bound > best_of_all_guess.total_count {
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
                let new_restrictions = Restriction::from(guess, pattern);
                dfs_with_cache(current + 1, &pattern_answers, &pattern_answers, new_restrictions, cache, use_limit, counter)
            } else {
                let new_restrictions = Restriction::from(guess, pattern);
                dfs_with_cache(current + 1, &pattern_answers, &filter_available_guesses(&new_restrictions, &availables), new_restrictions, cache, use_limit, counter)
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

    cache  
        .lock()
        .unwrap()
        .entry(restrictions)
        .or_insert_with(BTreeMap::new)
        .entry(availables.to_owned())
        .or_insert_with(BTreeMap::new)
        .insert(current, best_of_all_guess.clone());

    best_of_all_guess
}

pub fn dfs<'a>(current: u8, answers: &BTreeSet<&'a str>, availables: &BTreeSet<&'a str>) -> Best<'a> {

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
    
    preprocess_by_guess
        .sort_by_cached_key(|(_, entropy, _)| *entropy);
    
    let length = preprocess_by_guess.len();
    let top_guesses:Vec<_> = preprocess_by_guess
        .into_iter()
        .take(limit(length))
        .collect();

    for (guess, entropy, groups) in top_guesses {

        let mut lower_bound = entropy;
        let mut current_guess = Best::init(guess, answers.len() as u32);

        if best_of_all_guess.has_result && current_guess.total_count + lower_bound > best_of_all_guess.total_count {
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


fn basic_dfs<'a>(current: u8, answers: &BTreeSet<&'a str>, availables: &BTreeSet<&'a str>) -> Best<'a> {
    let mut best_of_all_guess = Best::new();
    for guess in availables {

        let groups = group_by_pattern(guess, &answers);
        
        if groups.len() == 1 && !groups.contains_key(&242) {
            continue
        }

        let mut current_guess = Best::init(guess, answers.len() as u32);

        for (pattern, pattern_answers) in  groups {
            let sub_result = if Checker::is_success_pattern(pattern) {
                Best {
                    has_result: true,
                    max_level: 0,
                    total_count: 0,
                    decision_tree: DecisionTree::new()
                }
            } else {
                let new_restrictions = Restriction::from(guess, pattern);
                basic_dfs(current + 1, &pattern_answers, &filter_available_guesses(&new_restrictions, &availables))
            };   
            current_guess.update(pattern, sub_result);
        }
        best_of_all_guess.better(current_guess);
    }

    best_of_all_guess
}