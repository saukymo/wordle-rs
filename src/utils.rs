use std::collections::{BTreeMap, BTreeSet};
use crate::common::Restriction;
use crate::game::Checker;

pub fn group_by_pattern<'a>(guess: &'a str, answers: &BTreeSet<&'a str>) -> BTreeMap<u8, BTreeSet<&'a str>>{
    let mut groups = BTreeMap::new();
    for answer in answers.into_iter() {
        let pattern = Checker::check(answer, guess);
        (*groups.entry(pattern).or_insert_with(BTreeSet::new)).insert(answer.clone());
    }
    groups
}


pub fn get_entropy(pattern: u8, length: u32) -> u32 {

    if pattern == 242 {
        return 0
    } else {
        return 2 * length - 1;
    }
    
}

pub fn get_lower_bound_level(length: usize) -> u8 {

    match length {
        1 => 1,
        2..=243 => 2,
        _ => 3
    }
}

pub fn get_entropy_sum<'a>(guess: &'a str, answers: &BTreeSet<&'a str>) -> (&'a str, u32, BTreeMap<u8, BTreeSet<&'a str>>) {
    let groups = group_by_pattern(guess, answers);

    let entropy = groups.iter().map(|(pattern, group)| {
        get_entropy(*pattern, group.len() as u32)
    }).sum();

    (guess, entropy, groups)
}

pub fn filter_available_guesses<'a> (restriction: &Restriction, words: &BTreeSet<&'a str>) -> BTreeSet<&'a str> {
    words.iter().filter(|word| {
        restriction.evaluate(word)
    }).cloned().collect()
}

pub fn filter_available_answers<'a> (guess: &'a str, pattern: u8, answers: &BTreeSet<&'a str>) -> BTreeSet<&'a str> {
    answers.iter().filter(|answer| {
        Checker::check(answer, guess) == pattern
    }).cloned().collect()
}

// limit 15 can get best results.
pub fn limit(_length: usize) -> usize {
    13
    // match length {
    //     0..=15 => length,
    //     _ => 15,
    // }
}