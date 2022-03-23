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
    // decision_tree: DecisionTree<'a>,
    current: &'a DecisionTree<'a>
}

impl<'a> Solver<'a> {
    pub fn guess(&self) -> String {
        self.current.guess()
    }
}

struct Evaluator<'a> {
    answers: &'a HashSet<&'a str>,
    words: &'a HashSet<&'a str>
}

impl<'a> Evaluator<'_> {
    pub fn evaluate(&self, decision_tree: DecisionTree, is_hard:bool) {
        let mut total = 0;
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
                if Checker::is_success_pattern(&pattern) {
                    break;
                }

                solver.current = DecisionTree::next(solver.current, &pattern);

                if is_hard {
                    restrictions = restrictions.merge(&Restriction::from(&guess, &pattern));
                    allowed = filter_available_guesses(&restrictions, &allowed);
                }

                assert!(turns < 10, "No answer less than 10.");
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

fn filter_available_guesses<'a> (restriction: &Restriction, words: &HashSet<&'a str>) -> HashSet<&'a str> {
    words.iter().filter(|word| {
        restriction.evaluate(word)
    }).cloned().collect()
}

#[derive(Debug, Clone, PartialEq)]
struct Best<'a> {
    max_level: u8,
    total_count: i16,
    decision_tree: DecisionTree<'a>
}

impl<'a> Best<'a> {
    pub fn new() -> Self {
        Best {
            max_level: 0,
            total_count: -1,
            decision_tree: DecisionTree::new()
        }
    }

    pub fn init(guess: &'a str, total_count: i16) -> Self {
        Best {
            max_level: 0,
            total_count: total_count,
            decision_tree: DecisionTree::from(guess, HashMap::new())
        }
    }

    pub fn better(&mut self, other: Best<'a>) {
        if self.total_count == -1 || self.total_count > other.total_count || (self.total_count == other.total_count && self.max_level > other.max_level + 1) {
            self.max_level = other.max_level + 1;
            self.total_count = other.total_count;
            self.decision_tree = other.decision_tree;
        }
    }

    pub fn update(&mut self, pattern: String, other: Best<'a>) {
        self.max_level = max(self.max_level, other.max_level);
        self.total_count += other.total_count;
        self.decision_tree.add_branch(pattern, other.decision_tree);
    }
}   

fn dfs<'a>(current: u8, answers: &HashSet<&'a str>, availables: &HashSet<&'a str>, restrictions: Restriction) -> Best<'a> {

    let valid_guesses = if answers.len() <= 3 {
        &answers
    } else {
        &availables
    };

    let mut best_of_all_guess = Best::new();

    for guess in valid_guesses.iter() {


        let groups = group_by_pattern(guess, &answers);
        if groups.len() == 1 && !groups.contains_key("GGGGG") {
            continue
        }
        let mut current_guess = Best::init(guess, answers.len() as i16);

        for (pattern, pattern_answers) in  groups {
            let sub_result = if Checker::is_success_pattern(&pattern) {
                Best {
                    max_level: 0,
                    total_count: 0,
                    decision_tree: DecisionTree::new()
                }
            } else {
                let new_restrictions = restrictions.merge(&Restriction::from(guess, &pattern));
                dfs(current + 1, &pattern_answers, &filter_available_guesses(&new_restrictions, &availables), new_restrictions)
            };
            
            current_guess.update(pattern, sub_result);
        }

        best_of_all_guess.better(current_guess);
    }

    best_of_all_guess
}

fn main() {
    let answers: HashSet<_> = include_str!("../data/answers.txt").lines().take(10).collect();
    let words: HashSet<_> = include_str!("../data/words.txt").lines().take(100).collect();

    let best = dfs(0, &answers, &words, Restriction::new());

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

        assert_eq!(filter_available_guesses(&restriction, &words), HashSet::from(["aazcc", "aaccz"]));
    }

    #[test]
    fn test_decision_tree() {
        let a = DecisionTree::from("fiveb", HashMap::from([]));
        let b = DecisionTree::from("salte", HashMap::from([]));

        let mut c = &DecisionTree::from("salet", HashMap::from([
            (String::from("BBBBB"), a),
            (String::from("GGGYY"), b),
        ]));
        
        assert_eq!(c.guess(), "salet");
        c = DecisionTree::next(&c, "GGGYY");
        assert_eq!(c.guess(), "salte");
        
    }

    #[test]
    fn test_single_search() {
        let best = dfs(0, &HashSet::from(["salet"]), &HashSet::from(["salet"]), Restriction::new());
        assert_eq!(best, Best {
            max_level: 1,
            total_count: 1,
            decision_tree: DecisionTree::from("salet", HashMap::from([
                (String::from("GGGGG"), DecisionTree::new())
            ]))
        })
    }

    #[test]
    fn test_a_few_search() {
        let answers = HashSet::from(["aback", 
        "abase",
        "abate",
        "abbey",
        "abbot",
        "abhor",
        "abide",
        "abled",
        "abode",
        "abort"]);
    
        let words = HashSet::from(["aback", 
        "abase",
        "abate",
        "abbey",
        "abbot",
        "abhor",
        "abide",
        "abled",
        "abode",
        "abort"]);

        let best = dfs(0, &answers, &words, Restriction::new());
        assert_eq!(best.max_level, 3);
        assert_eq!(best.total_count, 21);
    }
}