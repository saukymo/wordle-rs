
use std::collections::BTreeSet;

pub mod game;
pub mod utils;
pub mod common;
pub mod wrapper;
pub mod dfs;

use game::Evaluator;
use wrapper::{parallel_wrapper, start_word_wrapper};


fn main() {
    let answers: BTreeSet<_> = include_str!("../data/answers.txt").lines().collect();
    let words: BTreeSet<_> = include_str!("../data/words.txt").lines().collect();

    // 1075, total 3587, max 6
    // 1200, total 4032 max 6, 10s
    // 1300, total 4412 max 6, 18.97s
    // 1400, total 4793 max 6, 37.64s
    let best = start_word_wrapper("salet", &answers, &words);


    // 40, total 108 max 4, 78s
    // 600, total 1875 max 5, 270.84s
    // let best = parallel_wrapper("salet", &answers, &words);

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
    use std::sync::{Arc, Mutex};
    use std::collections::{BTreeMap, BTreeSet};

    use crate::utils::*;
    use crate::game::{Checker, Evaluator};
    use crate::common::{Restriction, Best, Cache, DecisionTree};
    use crate::wrapper::{start_word_wrapper, parallel_wrapper};
    use crate::dfs::{dfs, dfs_with_cache};    


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
        let best = dfs_with_cache(0, &BTreeSet::from(["salet"]), &BTreeSet::from(["salet"]), Restriction::new(), &Arc::new(Mutex::new(Cache::new())));
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
    fn test_a_few_search_with_cache() {
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

        let best = dfs_with_cache(0, &answers, &words, Restriction::new(), &Arc::new(Mutex::new(Cache::new())));
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
    fn test_a_few_search_without_cache() {
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

        let best = start_word_wrapper("salet", &answers, &words);
        assert_eq!(best.has_result, true);
        assert_eq!(best.max_level, 3);
        assert_eq!(best.total_count, 23); 

        let evaluator = Evaluator {
            answers: &answers,
            words: &words
        };

        evaluator.evaluate(best.decision_tree, true);
    }

    #[test]
    fn test_a_few_search_with_parallel_wrapper() {
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

        let best = parallel_wrapper("salet", &answers, &words);
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