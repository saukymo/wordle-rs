use std::collections::HashMap;

#[derive(Debug)]
pub struct DecisionTree<'a> {
    guess: &'a str,
    branch: HashMap<&'a str, DecisionTree<'a>>
}

impl<'a> DecisionTree<'a> {
    pub fn new(guess:&'a str, branch:HashMap<&'a str, DecisionTree<'a>>) -> Self {
        DecisionTree {
            guess,
            branch
        }
    }

    pub fn guess(&self) -> String {
        self.guess.to_string()
    }

    pub fn next (
        current: &'a DecisionTree,
        pattern: &str
    ) -> &'a DecisionTree<'a> {
        current.branch.get(pattern).unwrap()
    }
}

