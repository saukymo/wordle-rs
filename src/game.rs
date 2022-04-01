use std::cmp::max;
use std::collections::{BTreeMap, BTreeSet};
use crate::common::{Restriction, DecisionTree};
use crate::utils::{filter_available_guesses, stat_color};

pub struct Checker {
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

pub struct Solver<'a> {
    // decision_tree: DecisionTree<'a>,
    current: &'a DecisionTree<'a>
}

impl<'a> Solver<'a> {
    pub fn guess(&self) -> String {
        self.current.guess()
    }
}

pub struct Evaluator<'a> {
    pub answers: &'a BTreeSet<&'a str>,
    pub words: &'a BTreeSet<&'a str>
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

                println!("{}: {}", turns, stat_color(&guess, pattern));
                
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