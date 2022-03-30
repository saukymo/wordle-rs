use std::cmp::max;
use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct Restriction {
    pub required_green: BTreeMap<usize, char>,
    pub required_yellow: BTreeMap<char, usize>
}

impl Restriction {
    pub fn new() -> Self {
        Restriction {
            required_green: BTreeMap::new(),
            required_yellow: BTreeMap::new()
        }
    }

    pub fn from(guess: &str, pattern: u8) -> Self {
        let mut required_green = BTreeMap::new();
        let mut required_yellow = BTreeMap::new();


        let mut current = pattern;
        for (i, c) in guess.chars().enumerate() {
            let p = current % 3;
            match p {
                2 => { required_green.insert(i, c); },
                1 => { *required_yellow.entry(c).or_insert(0) += 1 },
                _ => ()
            }

            current /= 3;
            if current == 0 {
                break
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

        let mut counter: BTreeMap<char, usize> = BTreeMap::new();
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


#[derive(Debug, Clone, PartialEq)]
pub struct Best<'a> {
    pub has_result: bool,
    pub max_level: u8,
    pub total_count: u32,
    pub decision_tree: DecisionTree<'a>
}

impl<'a> Best<'a> {
    pub fn new() -> Self {
        Best {
            has_result: false,
            max_level: 0,
            total_count: u32::MAX,
            decision_tree: DecisionTree::new()
        }
    }

    pub fn init(guess: &'a str, total_count: u32) -> Self {
        Best {
            has_result: true,
            max_level: 0,
            total_count: total_count,
            decision_tree: DecisionTree::from(guess, BTreeMap::new())
        }
    }

    pub fn better(&mut self, other: Best<'a>) {
        if !other.has_result {
            return;
        }

        if self.total_count > other.total_count || (self.total_count == other.total_count && self.max_level > other.max_level + 1) {
            self.max_level = other.max_level + 1;
            self.total_count = other.total_count;
            self.decision_tree = other.decision_tree;
            self.has_result = true;
        }
    }

    pub fn update(&mut self, pattern: u8, other: Best<'a>) {
        if !other.has_result {
            return;
        }

        self.max_level = max(self.max_level, other.max_level);
        self.total_count += other.total_count;
        self.decision_tree.add_branch(pattern, other.decision_tree);
        self.has_result = true;
    }
}   

pub type Cache<'a> = BTreeMap<Restriction, BTreeMap<BTreeSet<&'a str>, BTreeMap<u8, Best<'a>>>>;
pub type Task<'a> = (u8, &'a str, u8);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionTree<'a> {
    pub guess: &'a str,
    pub branch: BTreeMap<u8, DecisionTree<'a>>
}

impl<'a> DecisionTree<'a> {
    pub fn new() -> Self {
        DecisionTree {
            guess: "",
            branch: BTreeMap::new()
        }
    }

    pub fn from(guess:&'a str, branch:BTreeMap<u8, DecisionTree<'a>>) -> Self {
        DecisionTree {
            guess,
            branch
        }
    }

    pub fn guess(&self) -> String {
        self.guess.to_string()
    }

    pub fn add_branch(&mut self, pattern: u8, tree: DecisionTree<'a>) {
        self.branch.insert(pattern, tree);
    }

    pub fn next (
        current: &'a DecisionTree,
        pattern: u8
    ) -> &'a DecisionTree<'a> {
        current.branch.get(&pattern).unwrap()
    }

    pub fn to_json(&self) -> Result<()> {
        println!("{}", serde_json::to_string(self)?);
        Ok(())
    }
}