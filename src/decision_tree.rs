use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct DecisionTree<'a> {
    pub guess: &'a str,
    branch: HashMap<String, DecisionTree<'a>>
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
}

