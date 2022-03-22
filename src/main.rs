use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
struct Restriction {
    required_green: HashMap<usize, char>,
    required_yellow: HashMap<char, usize>
}

impl Restriction {
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

    pub fn merge(&mut self, other: &Restriction) {
        for (pos, c) in other.required_green.iter() {
            self.required_green.insert(*pos, *c);
        }

        for (c, other_count) in other.required_yellow.iter() {
            let self_count = self.required_yellow.entry(*c).or_insert(0);
            *self_count = max(*self_count, *other_count)
        }
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


fn group_by_pattern<'a>(guess: &'a str, answers: &HashSet<&'a str>) -> HashMap<String, HashSet<&'a str>>{
    let mut groups = HashMap::new();
    for answer in answers.into_iter() {
        let pattern = Checker::check(answer, guess);
        (*groups.entry(pattern).or_insert_with(HashSet::new)).insert(answer.clone());
    }
    groups
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

        restriction_a.merge(&restriction_b);
        assert_eq!(restriction_a, Restriction{
            required_green: HashMap::from([(0, 'a'), (1, 'a')]),
            required_yellow: HashMap::from([('c', 2)]),
        });

        assert_eq!(restriction_a.evaluate("aazcc"), true);
        assert_eq!(restriction_a.evaluate("aaccz"), true);
        assert_eq!(restriction_a.evaluate("azbcc"), false);
        assert_eq!(restriction_a.evaluate("aabbc"), false);

    }
}