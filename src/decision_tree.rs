use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionTree<'a> {
    pub guess: &'a str,
    pub branch: HashMap<String, DecisionTree<'a>>
}

impl<'a> DecisionTree<'a> {
    pub fn new() -> Self {
        DecisionTree {
            guess: "",
            branch: HashMap::new()
        }
    }

    pub fn from(guess:&'a str, branch:HashMap<String, DecisionTree<'a>>) -> Self {
        DecisionTree {
            guess,
            branch
        }
    }

    pub fn guess(&self) -> String {
        self.guess.to_string()
    }

    pub fn add_branch(&mut self, pattern: String, tree: DecisionTree<'a>) {
        self.branch.insert(pattern, tree);
    }

    pub fn next (
        current: &'a DecisionTree,
        pattern: &str
    ) -> &'a DecisionTree<'a> {
        current.branch.get(pattern).unwrap()
    }

    pub fn to_json(&self) -> Result<()> {
        println!("{}", serde_json::to_string(self)?);
        Ok(())
    }
}

