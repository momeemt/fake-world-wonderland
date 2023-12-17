use std::collections::{HashSet, HashMap};

use crate::{fsa::{State, NFATransition, EpsilonTransition, NFA}, regexp::RegExp};

pub struct NFAConstructor {
    state_counter: State,
}

impl NFAConstructor {
    pub fn new() -> Self {
        Self { state_counter: 0 }
    }

    pub fn new_state(&mut self) -> State {
        self.state_counter += 1;
        self.state_counter
    }

    pub fn simple_dict_union(&self, d1: &NFATransition, d2: &NFATransition) -> NFATransition {
        let mut result = NFATransition::new();
        for (state, trans) in d1.iter().chain(d2) {
            let entry = result.entry(*state).or_insert_with(HashMap::new);
            for (ch, states) in trans {
                let entry_set = entry.entry(*ch).or_insert_with(HashSet::new);
                entry_set.extend(states.iter().cloned());
            }
        }
        result
    }

    pub fn eps_union(&self, e1: &EpsilonTransition, e2: &EpsilonTransition) -> EpsilonTransition {
        let mut result = EpsilonTransition::new();
        for (state, states) in e1.iter().chain(e2) {
            let entry = result.entry(*state).or_insert_with(HashSet::new);
            entry.extend(states.iter().cloned());
        }
        result
    }

    pub fn nfa_trans_union(&self, t1: &NFATransition, t2: &NFATransition) -> NFATransition {
        self.simple_dict_union(t1, t2)
    }

    pub fn end_state(&self, n: &NFA) -> Option<State> {
        if n.finals.len() == 1 {
            n.finals.iter().cloned().next()
        } else {
            None
        }
    }

    pub fn rx_to_nfa(&mut self, rx: &RegExp, alphabet: &HashSet<char>) -> Option<NFA> {
        match rx {
            RegExp::Char(ch) => {
                let start = self.new_state();
                let end = self.new_state();
                let mut trans = NFATransition::new();
                trans.insert(start, HashMap::from([(*ch, HashSet::from([end]))]));
                Some(NFA {
                    transition: trans,
                    epsilon_transition: EpsilonTransition::new(),
                    start,
                    finals: HashSet::from([end]),
                })
            },
            RegExp::Any => {
                let start = self.new_state();
                let end = self.new_state();
                let mut trans = NFATransition::new();
                let mut state_trans = HashMap::new();
                for &ch in alphabet {
                    state_trans.insert(ch, HashSet::from([end]));
                }
                trans.insert(start, state_trans);
                Some(NFA {
                    transition: trans,
                    epsilon_transition: EpsilonTransition::new(),
                    start,
                    finals: HashSet::from([end]),
                })
            },
            RegExp::Empty => {
                let start = self.new_state();
                let finals = HashSet::from([start]);
                Some(NFA {
                    transition: NFATransition::new(),
                    epsilon_transition: EpsilonTransition::new(),
                    start,
                    finals,
                })
            },
            RegExp::Seq { left, right } => {
                let l_nfa = self.rx_to_nfa(left, alphabet)?;
                let r_nfa = self.rx_to_nfa(right, alphabet)?;
                let l_end = self.end_state(&l_nfa)?;
                let eps_trans = self.eps_union(
                    &l_nfa.epsilon_transition, 
                    &HashMap::from([(l_end, HashSet::from([r_nfa.start]))])
                );
                Some(NFA {
                    transition: self.nfa_trans_union(&l_nfa.transition, &r_nfa.transition),
                    epsilon_transition: self.eps_union(&eps_trans, &r_nfa.epsilon_transition),
                    start: l_nfa.start,
                    finals: r_nfa.finals,
                })
            },
            RegExp::Or { left, right } => {
                let start = self.new_state();
                let l_nfa = self.rx_to_nfa(left, alphabet)?;
                let r_nfa = self.rx_to_nfa(right, alphabet)?;
                let end = self.new_state();
                let eps_trans = HashMap::from([
                    (start, HashSet::from([l_nfa.start, r_nfa.start])),
                    (self.end_state(&l_nfa)?, HashSet::from([end])),
                    (self.end_state(&r_nfa)?, HashSet::from([end])),
                ]);
                Some(NFA {
                    transition: self.nfa_trans_union(&l_nfa.transition, &r_nfa.transition),
                    epsilon_transition: self.eps_union(&eps_trans, &self.eps_union(&l_nfa.epsilon_transition, &r_nfa.epsilon_transition)),
                    start,
                    finals: HashSet::from([end]),
                })
            },
            RegExp::Repeat(reg) => {
                let start = self.new_state();
                let reg_nfa = self.rx_to_nfa(reg, alphabet)?;
                let end = self.new_state();
                let reg_end = self.end_state(&reg_nfa)?;
                let eps_trans = HashMap::from([
                    (start, HashSet::from([reg_nfa.start, end])),
                    (reg_end, HashSet::from([reg_nfa.start, end])),
                ]);
                Some(NFA {
                    transition: reg_nfa.transition,
                    epsilon_transition: self.eps_union(&eps_trans, &reg_nfa.epsilon_transition),
                    start,
                    finals: HashSet::from([end]),
                })
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{regexp::RegExp, rx_to_fsa::NFAConstructor, fsa::State};
    use std::collections::HashSet;

    #[test]
    fn test_rx_to_nfa_and_nfa_to_dfa() {
        let mut nfa_constructor = NFAConstructor::new();
        let alphabet = HashSet::from(['a', 'b']);

        let rx = RegExp::Seq {
            left: Box::new(RegExp::Or {
                left: Box::new(RegExp::Char('a')),
                right: Box::new(RegExp::Empty),
            }),
            right: Box::new(RegExp::Char('b')),
        };

        let nfa = nfa_constructor.rx_to_nfa(&rx, &alphabet).expect("Failed to convert RegExp to NFA");
        let dfa = nfa.to_dfa();
        let dfa_states: HashSet<State> = dfa.transition.keys().cloned()
            .chain(dfa.transition.values().flat_map(|trans| trans.values().cloned()))
            .collect();
        assert_eq!(dfa_states.len(), 4, "DFA should have 4 states");
    }
}
