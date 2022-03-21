use std::collections::HashMap;
use std::collections::HashSet;

struct Checker {
}

impl Checker {
    pub fn check(target: &str, guess: &str) -> String {
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

    pub fn is_success_pattern(pattern: &str) -> bool {
        pattern == "GGGGG"
    }
}

struct Solver<'a> {
    answers:HashSet<&'a str>
}

impl<'a> Solver<'a> {
    pub fn guess(&self) -> &str{
        self.answers.iter().next().unwrap()
    }

    pub fn update(&mut self, guess: &str, pattern: &str) {
        self.answers.retain(|answer| {
            Checker::check(answer, &guess) == pattern
        });
    }
}

struct Evaluator<'a> {
    answers: &'a HashSet<&'a str>
}

impl<'a> Evaluator<'_> {
    pub fn evaluate(&self) {
        let mut total = 0;
        for answer in self.answers.iter() {
            let mut solver = Solver {
                answers: self.answers.into_iter().cloned().collect()
            };

            let mut turns = 0;
            loop {
                let guess = solver.guess().to_string();
                let pattern = Checker::check(answer, &guess);

                turns += 1;
                if Checker::is_success_pattern(&pattern) {
                    break;
                }

                solver.update(&guess, &pattern)
            }

            total += turns;
        }

        println!("{:}: {:}", total, total as f32 / self.answers.len() as f32);
    }
}

fn main() {
    let answers: HashSet<_> = include_str!("../data/answers.txt").lines().collect();
    let words: HashSet<_> = include_str!("../data/words.txt").lines().collect();

    assert_eq!(answers.len(), 2315);
    assert_eq!(words.len(), 12972);

    let evaluator = Evaluator {
        answers: &answers
    };

    evaluator.evaluate();
}

#[cfg(test)]
mod tests {
    use crate::Checker;

    #[test]
    fn basic_check() {
        assert_eq!(Checker::check("admin", "crash"), "BBYBB");
        assert_eq!(Checker::check("abbbb", "caccc"), "BYBBB");
        assert_eq!(Checker::check("babbb", "caccc"), "BGBBB");
        assert_eq!(Checker::check("aabbb", "ccaac"), "BBYYB");
        assert_eq!(Checker::check("aabbb", "cccac"), "BBBYB");
        assert_eq!(Checker::check("aabbb", "caccc"), "BGBBB");
        assert_eq!(Checker::check("baabb", "acaac"), "YBGBB");
        assert_eq!(Checker::check("aaaar", "error"), "BBBBG");
    }

    #[test]
    fn check_if_success() {
        assert_eq!(Checker::is_success_pattern("BBYBB"), false);
        assert_eq!(Checker::is_success_pattern("GGGGG"), true);
    }
}