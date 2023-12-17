use std::collections::{HashMap, HashSet};

pub type State = i32;
pub type NFATransition = HashMap<State, HashMap<char, HashSet<State>>>;
pub type EpsilonTransition = HashMap<State, HashSet<State>>;
pub type DFATransition = HashMap<State, HashMap<char, State>>;

#[derive(Debug, Clone, PartialEq)]
pub struct NFA {
    pub transition: NFATransition,
    pub epsilon_transition: EpsilonTransition,
    pub start: State,
    pub finals: HashSet<State>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DFA {
    pub transition: DFATransition,
    pub start: State,
    pub finals: HashSet<State>,
}

impl NFA {
    // states内の各状態からε遷移した先の集合
    pub fn epsilon_closure_step(&self, states: &HashSet<State>) -> HashSet<State> {
        let mut result = HashSet::new();
        for state in states {
            if self.epsilon_transition.contains_key(&state) {
                for s in self.epsilon_transition[&state].clone().into_iter() {
                    result.insert(s);
                }
            }
            result.insert(*state);
        }
        result
    }

    pub fn get_epsilon_closure(&self, states: HashSet<State>) -> HashSet<State> {
        fn inner(nfa: &NFA, states: HashSet<State>) -> HashSet<State> {
            let new = nfa.epsilon_closure_step(&states);
            if new == states {
                return new;
            }
            inner(nfa, new)
        }
        inner(self, states)
    }

    // currentからsymで遷移した先の集合
    pub fn transit(&self, current: HashSet<State>, sym: char) -> HashSet<State> {
        let mut result = HashSet::new();
        for state in current {
            if self.transition.contains_key(&state) && self.transition[&state].contains_key(&sym) {
                for s in self.transition[&state][&sym].clone().into_iter() {
                    result.insert(s);
                }
            }
        }
        self.get_epsilon_closure(result)
    }

    pub fn is_final(&self, states: HashSet<State>) -> bool {
        for state in states {
            if self.finals.contains(&state) {
                return true;
            }
        }
        false
    }

    pub fn try_accept(&self, code: &str) -> bool {
        let mut current = HashSet::from([self.start]);
        for ch in code.chars() {
            current = self.transit(current, ch);
        }
        self.is_final(current)
    }

    pub fn to_dfa(&self) -> DFA {
        let mut new_states: Vec<HashSet<State>> =
            vec![self.get_epsilon_closure(HashSet::from([self.start]))];
        let mut trans_dict: DFATransition = HashMap::new();
        let mut src = 0;

        let alphabet: HashSet<char> = self
            .transition
            .values()
            .flat_map(|trans| trans.keys().cloned())
            .collect();

        while src < new_states.len() {
            let cur = new_states[src].clone();
            let mut src_trans = HashMap::new();

            for &c in &alphabet {
                let c_next: HashSet<State> = self.get_epsilon_closure(self.transit(cur.clone(), c));

                let dest = if let Some(pos) = new_states.iter().position(|state| state == &c_next) {
                    pos
                } else {
                    new_states.push(c_next.clone());
                    new_states.len() - 1
                };

                src_trans.insert(c, dest as State);
            }

            trans_dict.insert(src as State, src_trans);
            src += 1;
        }

        let finals: HashSet<State> = new_states
            .iter()
            .enumerate()
            .filter_map(|(i, states)| {
                if self.is_final(states.clone()) {
                    Some(i as State)
                } else {
                    None
                }
            })
            .collect();

        DFA {
            transition: trans_dict,
            start: 0,
            finals,
        }
    }
}

impl DFA {
    pub fn try_accept(&self, code: &str) -> bool {
        let mut current = self.start;
        for ch in code.chars() {
            if let Some(next_state) = self
                .transition
                .get(&current)
                .and_then(|trans| trans.get(&ch))
            {
                current = *next_state;
            } else {
                return false;
            }
        }
        self.finals.contains(&current)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::{DFA, NFA};

    #[test]
    fn test_nfa1() {
        let nfa = NFA {
            transition: vec![(0, 'a', 1), (1, 'b', 2), (2, 'c', 3)]
                .into_iter()
                .fold(HashMap::new(), |mut acc, (state, ch, next_state)| {
                    acc.entry(state)
                        .or_insert_with(HashMap::new)
                        .entry(ch)
                        .or_insert_with(HashSet::new)
                        .insert(next_state);
                    acc
                }),
            epsilon_transition: vec![(0, 1), (1, 2), (2, 3)].into_iter().fold(
                HashMap::new(),
                |mut acc, (state, next_state)| {
                    acc.entry(state)
                        .or_insert_with(HashSet::new)
                        .insert(next_state);
                    acc
                },
            ),
            start: 0,
            finals: vec![3].into_iter().collect(),
        };
        assert!(nfa.try_accept("abc"));
        assert!(nfa.try_accept("ab"));
        assert!(!nfa.try_accept("abcd"));
    }

    #[test]
    fn test_nfa2() {
        let nfa = NFA {
            transition: vec![
                (0, 'a', 1),
                (1, 'b', 2),
                (2, 'c', 3),
                (3, 'd', 4),
                (4, 'e', 5),
                (5, 'f', 6),
            ]
            .into_iter()
            .fold(HashMap::new(), |mut acc, (state, ch, next_state)| {
                acc.entry(state)
                    .or_insert_with(HashMap::new)
                    .entry(ch)
                    .or_insert_with(HashSet::new)
                    .insert(next_state);
                acc
            }),
            epsilon_transition: vec![(0, 1), (2, 3), (5, 6)].into_iter().fold(
                HashMap::new(),
                |mut acc, (state, next_state)| {
                    acc.entry(state)
                        .or_insert_with(HashSet::new)
                        .insert(next_state);
                    acc
                },
            ),
            start: 0,
            finals: vec![6].into_iter().collect(),
        };
        assert!(nfa.try_accept("abcdef"));
        assert!(nfa.try_accept("abcde"));
        assert!(!nfa.try_accept("abcdeg"));
    }

    #[test]
    fn test_nfa3() {
        let nfa = NFA {
            transition: vec![
                (0, 'a', 1),
                (0, 'a', 2),
                (1, 'b', 3),
                (2, 'b', 3),
                (3, 'c', 4),
            ]
            .into_iter()
            .fold(HashMap::new(), |mut acc, (state, ch, next_state)| {
                acc.entry(state)
                    .or_insert_with(HashMap::new)
                    .entry(ch)
                    .or_insert_with(HashSet::new)
                    .insert(next_state);
                acc
            }),
            epsilon_transition: vec![(0, 1), (0, 2)].into_iter().fold(
                HashMap::new(),
                |mut acc, (state, next_state)| {
                    acc.entry(state)
                        .or_insert_with(HashSet::new)
                        .insert(next_state);
                    acc
                },
            ),
            start: 0,
            finals: vec![4].into_iter().collect(),
        };
        assert!(nfa.try_accept("abc"));
        assert!(!nfa.try_accept("bbc"));
        assert!(!nfa.try_accept("ab"));
        assert!(!nfa.try_accept("abcd"));
    }

    #[test]
    fn test_dfa1() {
        let dfa = DFA {
            transition: vec![(0, 'a', 1), (1, 'b', 2), (2, 'c', 3)]
                .into_iter()
                .fold(HashMap::new(), |mut acc, (state, ch, next_state)| {
                    acc.entry(state)
                        .or_insert_with(HashMap::new)
                        .entry(ch)
                        .or_insert(next_state);
                    acc
                }),
            start: 0,
            finals: vec![3].into_iter().collect(),
        };
        assert!(dfa.try_accept("abc"));
        assert!(!dfa.try_accept("ab"));
        assert!(!dfa.try_accept("abcd"));
    }
}
