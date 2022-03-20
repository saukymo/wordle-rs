use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;

struct Checker {
}

impl Checker {
    pub fn check(target: &String, guess: &String) -> String {
        let mut pattern: String = String::from("");
        let mut freq = HashMap::<char, usize>::new();
        for (guess_c, target_c) in guess.chars().zip(target.chars()) {
            if guess_c != target_c {
                let counter = freq.entry(target_c).or_insert(0);
                *counter += 1;
            }
        }

        for (guess_c, target_c) in guess.chars().zip(target.chars()) {
            if guess_c == target_c {
                pattern += "G";
            } else {
                let counter = freq.entry(guess_c).or_insert(0);
                if *counter > 0 {
                    pattern += "Y";
                    *counter -= 1;
                } else {
                    pattern += "B";
                }
            }
        }
        pattern
    }

    pub fn is_success_pattern(pattern: &String) -> bool {
        pattern == "GGGGG"
    }
}

struct Solver {
    answers: Vec<String>
}

impl Solver {
    pub fn guess(&self) -> &String{
        let mut rng = thread_rng();
        self.answers.choose(&mut rng).unwrap()
    }

    pub fn update(&self, guess: &String, pattern: String) -> Vec<String> {
        // let mut filtered_answers: Vec<String> = Vec::new();

        // for answer in self.answers.iter() {
        //     if Checker::check(answer, guess) == pattern {
        //         filtered_answers.push(answer.to_string())
        //     }
        // }

        // filtered_answers
        self.answers.iter().filter(|answer| {
            Checker::check(answer, guess) == pattern
        }).map(|x| x.to_string()).collect()
    }
}

struct Evaluator {
    answers: Vec<String>
}

impl Evaluator {
    pub fn evaluate(&self) {
        let mut total = 0;
        for answer in self.answers.iter() {
            let mut solver = Solver {
                answers: self.answers.clone()
            };

            let mut turns = 0;

            loop {
                let guess = solver.guess();
                let pattern = Checker::check(&answer, guess);

                turns += 1;
                if Checker::is_success_pattern(&pattern) {
                    break;
                }

                solver.answers = solver.update(guess, pattern);
            }

            total += turns;
        }

        println!("{:}: {:}", total, total as f32 / self.answers.len() as f32);
    }
}

fn main() {
    let answers: Vec<String> = include_str!("../data/answers.txt").lines().map(|x| x.to_string()).collect();
    let words: Vec<String> = include_str!("../data/words.txt").lines().map(|x| x.to_string()).collect();

    assert_eq!(answers.len(), 2315);
    assert_eq!(words.len(), 12972);

    let evaluator = Evaluator {
        answers
    };

    evaluator.evaluate();
}

#[cfg(test)]
mod tests {
    use crate::Checker;

    #[test]
    fn basic_check() {
        assert_eq!(Checker::check(&String::from("admin"), &String::from("crash")), String::from("BBYBB"));
        assert_eq!(Checker::check(&String::from("abbbb"), &String::from("caccc")), String::from("BYBBB"));
        assert_eq!(Checker::check(&String::from("babbb"), &String::from("caccc")), String::from("BGBBB"));
        assert_eq!(Checker::check(&String::from("aabbb"), &String::from("ccaac")), String::from("BBYYB"));
        assert_eq!(Checker::check(&String::from("aabbb"), &String::from("cccac")), String::from("BBBYB"));
        assert_eq!(Checker::check(&String::from("aabbb"), &String::from("caccc")), String::from("BGBBB"));
        assert_eq!(Checker::check(&String::from("baabb"), &String::from("acaac")), String::from("YBGBB"));
        assert_eq!(Checker::check(&String::from("aaaar"), &String::from("error")), String::from("BBBBG"));
    }

    #[test]
    fn check_if_success() {
        assert_eq!(Checker::is_success_pattern(&String::from("BBYBB")), false);
        assert_eq!(Checker::is_success_pattern(&String::from("GGGGG")), true);
    }
}