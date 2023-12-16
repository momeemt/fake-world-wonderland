use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum RegExp {
    Char(char),
    Any,
    Empty,
    Seq {
        left: Box<RegExp>,
        right: Box<RegExp>,
    },
    Or {
        left: Box<RegExp>,
        right: Box<RegExp>,
    },
    Repeat(Box<RegExp>),
}

impl RegExp {
    fn repeat_match(&self, input: &str, pos: usize, acc: HashSet<usize>) -> Option<HashSet<usize>> {
        let mut next = HashSet::new();
        let res = self._match(input, pos);
        if res.is_none() {
            return Some(acc);
        }
        for q in res? {
            if !acc.contains(&q) {
                next.insert(q);
            }
        }

        if next.is_empty() {
            Some(acc) 
        } else {
            match (acc, next) {
                (acc, next) => {
                    let mut new_acc = HashSet::new();
                    for q in acc {
                        new_acc.insert(q);
                    }
                    for q in next {
                        new_acc.insert(q);
                    }
                    self.repeat_match(input, pos + 1, new_acc)
                }
            }
        }
    }

    pub fn _match(&self, input: &str, pos: usize) -> Option<HashSet<usize>> {
        match self {
            RegExp::Char(c) => {
                if pos < input.len() && input.chars().nth(pos)? == *c {
                    return Some(HashSet::from([pos + 1]));
                }
            },
            RegExp::Any => {
                if pos < input.len() {
                    return Some(HashSet::from([pos + 1]));
                }
            },
            RegExp::Empty => {
                if pos <= input.len() {
                    return Some(HashSet::from([pos]));
                }
            },
            RegExp::Seq { left, right } => {
                let mut result = HashSet::new();
                for pos_left in left._match(input, pos)? {
                    for pos_right in right._match(input, pos_left)? {
                        result.insert(pos_right);
                    }
                }
                return Some(result);
            },
            RegExp::Or { left, right } => {
                let left_result = left._match(input, pos);
                let right_result = right._match(input, pos);
                match (left_result, right_result) {
                    (Some(left_result), Some(right_result)) => {
                        let mut result = HashSet::new();
                        for pos in left_result {
                            result.insert(pos);
                        }
                        for pos in right_result {
                            result.insert(pos);
                        }
                        return Some(result);
                    },
                    (Some(left_result), None) => {
                        let mut result = HashSet::new();
                        for pos in left_result {
                            result.insert(pos);
                        }
                        return Some(result);
                    },
                    (None, Some(right_result)) => {
                        let mut result = HashSet::new();
                        for pos in right_result {
                            result.insert(pos);
                        }
                        return Some(result);
                    },
                    (None, None) => {
                        return None;
                    }
                }
            },
            RegExp::Repeat(reg) => {
                let initial_pos = HashSet::from([pos]);
                return reg.repeat_match(input, pos, initial_pos);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::RegExp;

    #[test]
    fn test_regexp_seq1() {
        let regexp = RegExp::Seq {
            left: Box::new(RegExp::Char('a')),
            right: Box::new(RegExp::Char('b')),
        };
        assert_eq!(regexp._match("ab", 0), Some([2].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_seq2() {
        let regexp = RegExp::Seq {
            left: Box::new(RegExp::Char('a')),
            right: Box::new(RegExp::Char('b')),
        };
        assert_eq!(regexp._match("ab", 1), None);
    }

    #[test]
    fn test_regexp_or1() {
        let regexp = RegExp::Or {
            left: Box::new(RegExp::Char('a')),
            right: Box::new(RegExp::Char('b')),
        };
        assert_eq!(regexp._match("a", 0), Some([1].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_or2() {
        let regexp = RegExp::Or {
            left: Box::new(RegExp::Char('a')),
            right: Box::new(RegExp::Char('b')),
        };
        assert_eq!(regexp._match("b", 0), Some([1].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_or3() {
        let regexp = RegExp::Or {
            left: Box::new(RegExp::Char('a')),
            right: Box::new(RegExp::Char('b')),
        };
        assert_eq!(regexp._match("c", 0), None);
    }

    #[test]
    fn test_regexp_repeat1() {
        let regexp = RegExp::Repeat(Box::new(RegExp::Char('a')));
        assert_eq!(regexp._match("a", 0), Some([0, 1].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_repeat2() {
        let regexp = RegExp::Repeat(Box::new(RegExp::Char('a')));
        assert_eq!(regexp._match("aa", 0), Some([0, 1, 2].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_repeat3() {
        let regexp = RegExp::Repeat(Box::new(RegExp::Char('a')));
        assert_eq!(regexp._match("aaa", 0), Some([0, 1, 2, 3].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_char1() {
        let regexp = RegExp::Char('a');
        assert_eq!(regexp._match("a", 0), Some([1].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_char2() {
        let regexp = RegExp::Char('a');
        assert_eq!(regexp._match("b", 0), None);
    }

    #[test]
    fn test_regexp_any1() {
        let regexp = RegExp::Any;
        assert_eq!(regexp._match("a", 0), Some([1].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_any2() {
        let regexp = RegExp::Any;
        assert_eq!(regexp._match("b", 0), Some([1].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_empty1() {
        let regexp = RegExp::Empty;
        assert_eq!(regexp._match("a", 0), Some([0].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_empty2() {
        let regexp = RegExp::Empty;
        assert_eq!(regexp._match("b", 0), Some([0].iter().copied().collect()));
    }

    #[test]
    fn test_regexp_empty3() {
        let regexp = RegExp::Empty;
        assert_eq!(regexp._match("", 0), Some([0].iter().copied().collect()));
    }
}
