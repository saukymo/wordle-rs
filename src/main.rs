use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;

pub mod decision_tree;
use decision_tree::DecisionTree;

#[derive(Debug, PartialEq, Clone)]
struct Restriction {
    required_green: HashMap<usize, char>,
    required_yellow: HashMap<char, usize>
}

impl Restriction {
    pub fn new() -> Self {
        Restriction {
            required_green: HashMap::new(),
            required_yellow: HashMap::new()
        }
    }

    pub fn from(guess: &str, pattern: &str) -> Self {
        let mut required_green: HashMap<usize, char> = HashMap::new();
        let mut required_yellow: HashMap<char, usize> = HashMap::new();

        for (i, (c, p)) in guess.chars().zip(pattern.chars()).enumerate() {
            match p {
                'G' => { required_green.insert(i, c); },
                'Y' => { *required_yellow.entry(c).or_insert(0) += 1 },
                _ => ()
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

        let mut counter: HashMap<char, usize> = HashMap::new();
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
    answers: &'a HashSet<&'a str>,
    words: &'a HashSet<&'a str>
}

impl<'a> Evaluator<'_> {
    pub fn evaluate(&self, is_hard:bool) {
        let mut total = 0;
        for answer in self.answers.iter() {
            let mut solver = Solver {
                answers: self.answers.into_iter().cloned().collect()
            };
            
            let mut allowed = self.words.clone();
            let mut restrictions = Restriction::new();

            let mut turns = 0;
            loop {
                let guess = solver.guess().to_string();

                assert_eq!(allowed.contains(guess.as_str()), true);

                let pattern = Checker::check(answer, &guess);

                turns += 1;
                if Checker::is_success_pattern(&pattern) {
                    break;
                }

                solver.update(&guess, &pattern);

                if is_hard {
                    restrictions = restrictions.merge(&Restriction::from(&guess, &pattern));
                    allowed = filter_available_guesses(&restrictions, allowed);
                }
            }

            total += turns;
        }

        println!("{:}: {:}", total, total as f32 / self.answers.len() as f32);
    }
}


fn group_by_pattern<'a>(guess: &'a str, answers: &HashSet<&'a str>) -> HashMap<String, HashSet<&'a str>>{
    let mut groups = HashMap::new();
    for answer in answers.into_iter() {
        let pattern = Checker::check(answer, guess);
        (*groups.entry(pattern).or_insert_with(HashSet::new)).insert(answer.clone());
    }
    groups
}

fn filter_available_guesses<'a> (restriction: &Restriction, words: HashSet<&'a str>) -> HashSet<&'a str> {
    words.iter().filter(|word| {
        restriction.evaluate(word)
    }).cloned().collect()
}

fn main() {
    let answers: HashSet<_> = include_str!("../data/answers.txt").lines().collect();
    let words: HashSet<_> = include_str!("../data/words.txt").lines().collect();

    assert_eq!(answers.len(), 2315);
    assert_eq!(words.len(), 12972);

    let evaluator = Evaluator {
        answers: &answers,
        words: &words
    };

    evaluator.evaluate(true);
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::collections::HashMap;
    use std::collections::HashSet;

    #[test]
    fn test_check() {
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
    fn test_if_success() {
        assert_eq!(Checker::is_success_pattern("BBYBB"), false);
        assert_eq!(Checker::is_success_pattern("GGGGG"), true);
    }

    #[test]
    fn test_group_by_pattern() {
        assert_eq!(group_by_pattern("salet", &HashSet::from(["sblet", "sclet", "zzzzz"])), HashMap::from([
           (String::from("GBGGG"), HashSet::from(["sblet", "sclet"])),
           (String::from("BBBBB"), HashSet::from(["zzzzz"])) 
        ]));
    }

    #[test]
    fn test_restriction() {
        let mut restriction_a= Restriction {
            required_green: HashMap::from([(1, 'a')]),
            required_yellow: HashMap::from([('c', 2)]),
        };
        
        let restriction_b = Restriction::from("azczz", "GBYBB");
        assert_eq!(restriction_b, Restriction{
            required_green: HashMap::from([(0, 'a')]),
            required_yellow: HashMap::from([('c', 1)]),
        });

        restriction_a = restriction_a.merge(&restriction_b);
        assert_eq!(restriction_a, Restriction{
            required_green: HashMap::from([(0, 'a'), (1, 'a')]),
            required_yellow: HashMap::from([('c', 2)]),
        });

        assert_eq!(restriction_a.evaluate("aazcc"), true);
        assert_eq!(restriction_a.evaluate("aaccz"), true);
        assert_eq!(restriction_a.evaluate("azbcc"), false);
        assert_eq!(restriction_a.evaluate("aabbc"), false);
    }

    #[test]
    fn test_filter_guesses() {
        let restriction = Restriction {
            required_green: HashMap::from([(0, 'a'), (1, 'a')]),
            required_yellow: HashMap::from([('c', 2)]),
        };

        let words = HashSet::from(["aazcc", "aaccz", "azbcc", "aabbc"]);

        assert_eq!(filter_available_guesses(&restriction, words), HashSet::from(["aazcc", "aaccz"]));
    }

    #[test]
    fn test_decision_tree() {
        let a = DecisionTree::new("fiveb", HashMap::from([]));
        let b = DecisionTree::new("salte", HashMap::from([]));

        let mut c = &DecisionTree::new("salet", HashMap::from([
            ("BBBBB", a),
            ("GGGYY", b),
        ]));
        
        assert_eq!(c.guess(), "salet");
        c = DecisionTree::next(&c, "GGGYY");
        assert_eq!(c.guess(), "salte");
        
    }
}