/// none: the highest priority to process, means can't include any one
/// all : the middle priority to process, means must include all
/// any: the lowest priority to process, means must include one

use std::collections::{HashMap, HashSet};

pub type ContextChecker = fn(contexts: &HashMap<String, String>,
                             none: &HashSet<String>,
                             all: &HashSet<String>,
                             any: &HashSet<String>) -> bool;

pub fn context_check(contexts: &HashMap<String, String>,
                     none: &HashSet<String>,
                     all: &HashSet<String>,
                     any: &HashSet<String>) -> bool {
    for exclude in none {
        if contexts.contains_key(exclude) {
            return false;
        }
    }
    for include in all {
        if !contexts.contains_key(include) {
            return false;
        }
    }
    if any.is_empty() {
        return true;
    }
    for o in any {
        if contexts.contains_key(o) {
            return true;
        }
    }
    false
}

pub type StateChecker = fn(status: &HashSet<String>,
                           none: &HashSet<String>,
                           all: &HashSet<String>,
                           any: &HashSet<String>) -> bool;

pub fn state_check(status: &HashSet<String>,
                   none: &HashSet<String>,
                   all: &HashSet<String>,
                   any: &HashSet<String>) -> bool {
    for exclude in none {
        if status.contains(exclude) {
            return false;
        }
    }
    for include in all {
        if !status.contains(include) {
            return false;
        }
    }
    if any.is_empty() {
        return true;
    }
    for o in any {
        if status.contains(o) {
            return true;
        }
    }
    false
}


#[cfg(test)]
mod demand_test {
    use super::*;

    #[test]
    fn status_check_test() {
        // check nothing
        assert_eq!(state_check(
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
        ), true);
        let mut states = HashSet::<String>::new();
        states.insert("a".to_string());
        assert_eq!(state_check(
            &states,
            &Default::default(),
            &Default::default(),
            &Default::default(),
        ), true);


        // check none
        let mut set = HashSet::<String>::new();
        set.insert("a,b".to_string());
        assert_eq!(state_check(
            &Default::default(),
            &set,
            &Default::default(),
            &Default::default(),
        ), true);
        let mut states = HashSet::<String>::new();
        states.insert("b".to_string());
        assert_eq!(state_check(
            &Default::default(),
            &set,
            &Default::default(),
            &Default::default(),
        ), false);
    }

    #[test]
    fn status_any() {}

    #[test]
    fn status_exclude() {}
}
