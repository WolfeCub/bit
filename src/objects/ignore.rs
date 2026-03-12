use std::fs;

use itertools::Itertools;

use crate::util::repo_root;

pub struct Ignore {
    patterns: Vec<Pattern>,
}

impl Ignore {
    pub fn build_from_disk() -> anyhow::Result<Ignore> {
        let root = repo_root()?;
        let contents = fs::read_to_string(root.join(".bitignore"))?;

        let patterns = contents
            .lines()
            .filter_map(|line| Pattern::parse(line))
            .collect();

        Ok(Ignore { patterns })
    }

    pub fn is_file_ignored(&self, path: &str, is_dir: bool) -> bool {
        let mut ignored = false;

        for pattern in &self.patterns {
            if pattern.matches(path, is_dir) {
                ignored = !pattern.is_negated;
            }
        }

        ignored
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Slash,
    Literal(String),
    /// * — any chars except '/'
    AnyFile,
    /// ** — any chars including '/'
    AnyPath,
    /// ? — single char except '/'
    AnyChar,
    CharClass {
        chars: Vec<char>,
        negated: bool,
    },
}

#[derive(Debug)]
struct Pattern {
    tokens: Vec<Token>,
    is_negated: bool,
    is_dir_only: bool,
    is_anchored: bool,
}

impl Pattern {
    fn parse(line: &str) -> Option<Self> {
        let rest = line.trim_end();

        if rest.is_empty() || rest.starts_with('#') {
            return None;
        }

        let (is_negated, rest) = match rest.strip_prefix('!') {
            Some(r) => (true, r),
            None => (false, rest),
        };

        let (prefix_slash, rest) = match rest.strip_prefix('/') {
            Some(r) => (true, r),
            None => (false, rest),
        };

        let (is_dir_only, rest) = match rest.strip_suffix('/') {
            Some(r) => (true, r),
            None => (false, rest),
        };

        let mut tokens = Vec::new();
        let mut chars = rest.chars().peekable();
        let mut is_anchored = prefix_slash;

        while chars.peek().is_some() {
            let literal = chars
                .peeking_take_while(|c| !matches!(c, '/' | '\\' | '*' | '?' | '['))
                .collect::<String>();
            if !literal.is_empty() {
                tokens.push(Token::Literal(literal));
            }

            match chars.next() {
                Some('/') => {
                    is_anchored = true;
                    tokens.push(Token::Slash);
                }
                Some('\\') => {
                    if let Some(escaped) = chars.next() {
                        tokens.push(Token::Literal(escaped.to_string()));
                    }
                }
                Some('*') => {
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        tokens.push(Token::AnyPath);
                    } else {
                        tokens.push(Token::AnyFile);
                    }
                }
                Some('?') => tokens.push(Token::AnyChar),
                Some('[') => {
                    let mut char_class = Vec::new();
                    let mut negated = false;
                    if chars.peek() == Some(&'!') {
                        chars.next();
                        negated = true;
                    }
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == ']' {
                            break;
                        }
                        char_class.push(c);
                    }
                    tokens.push(Token::CharClass {
                        chars: char_class,
                        negated,
                    });
                }
                None => break,
                _ => unreachable!("Should not reach here due to peeking_take_while"),
            }
        }

        Some(Self {
            tokens,
            is_negated,
            is_dir_only,
            is_anchored,
        })
    }

    pub fn matches(&self, path: &str, is_dir: bool) -> bool {
        if self.is_dir_only && !is_dir {
            return false;
        }
        if self.is_anchored {
            match_tokens(&self.tokens, path)
        } else {
            path.split('/').any(|s| match_tokens(&self.tokens, s))
        }
    }
}

fn match_tokens(tokens: &[Token], input: &str) -> bool {
    let Some((token, rest)) = tokens.split_first() else {
        return input.is_empty();
    };
    match token {
        Token::Literal(lit) => input
            .strip_prefix(lit.as_str())
            .is_some_and(|t| match_tokens(rest, t)),
        Token::Slash => input
            .strip_prefix('/')
            .is_some_and(|t| match_tokens(rest, t)),
        Token::AnyChar => input
            .chars()
            .next()
            .is_some_and(|c| c != '/' && match_tokens(rest, &input[1..])),
        Token::AnyFile => {
            // * matches any number of not '/' characters, so find how far we can
            // consume before hitting a slash (or end of input), then try each
            // possible match length from 0 up to that limit
            let max = input.find('/').unwrap_or(input.len());
            (0..=max).any(|i| match_tokens(rest, &input[i..]))
        }
        Token::AnyPath => {
            // `**/` and `**` should behave the same, so skip a leading slash in the remainder
            let inner = match rest {
                [Token::Slash, r @ ..] => r,
                _ => rest,
            };

            // Try matching at the current position (** matches zero path segments),
            // then try again after each '/' (** matches one or more path segments)
            let match_here = match_tokens(inner, input);
            let match_after_slash = input
                .match_indices('/')
                .any(|(i, _)| match_tokens(inner, &input[i + 1..]));

            match_here || match_after_slash
        }
        Token::CharClass { chars, negated } => {
            let Some(c) = input.chars().next() else {
                return false;
            };
            let matches_class = char_in_class(c, chars) != *negated;
            matches_class && match_tokens(rest, &input[c.len_utf8()..])
        }
    }
}

fn char_in_class(c: char, class: &[char]) -> bool {
    let mut i = 0;
    while i < class.len() {
        if class.get(i + 1) == Some(&'-') && i + 2 < class.len() {
            // Range pattern like 'a-z': check if c falls between the two bounds
            if c >= class[i] && c <= class[i + 2] {
                return true;
            }
            i += 3;
        } else {
            // Literal character: check for exact match
            if c == class[i] {
                return true;
            }
            i += 1;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let pattern = Pattern::parse("*.log").unwrap();
        assert_eq!(pattern.tokens.len(), 2);
        let expected = vec![Token::AnyFile, Token::Literal(".log".to_string())];
        assert_eq!(pattern.tokens, expected);
        assert!(pattern.is_negated == false);
        assert!(pattern.is_dir_only == false);
        assert!(pattern.is_anchored == false);
    }

    #[test]
    fn parse_anchored() {
        let pattern = Pattern::parse("foo/bar/it").unwrap();
        assert_eq!(pattern.tokens.len(), 5);
        let expected = vec![
            Token::Literal("foo".to_string()),
            Token::Slash,
            Token::Literal("bar".to_string()),
            Token::Slash,
            Token::Literal("it".to_string()),
        ];
        assert_eq!(pattern.tokens, expected);
        assert!(pattern.is_negated == false);
        assert!(pattern.is_dir_only == false);
        assert!(pattern.is_anchored == true);
    }

    #[test]
    fn parse_negated() {
        let pattern = Pattern::parse("!blah").unwrap();
        assert_eq!(pattern.tokens.len(), 1);
        let expected = vec![Token::Literal("blah".to_string())];
        assert_eq!(pattern.tokens, expected);
        assert!(pattern.is_negated == true);
        assert!(pattern.is_dir_only == false);
        assert!(pattern.is_anchored == false);
    }

    #[test]
    fn parse_dir_only() {
        let pattern = Pattern::parse("blah/").unwrap();
        assert_eq!(pattern.tokens.len(), 1);
        let expected = vec![Token::Literal("blah".to_string())];
        assert_eq!(pattern.tokens, expected);
        assert!(pattern.is_negated == false);
        assert!(pattern.is_dir_only == true);
        assert!(pattern.is_anchored == false);
    }

    #[test]
    fn parse_anchored_negated_dir_only() {
        let pattern = Pattern::parse("!foo/bar/").unwrap();
        assert_eq!(pattern.tokens.len(), 3);
        let expected = vec![
            Token::Literal("foo".to_string()),
            Token::Slash,
            Token::Literal("bar".to_string()),
        ];
        assert_eq!(pattern.tokens, expected);
        assert!(pattern.is_negated == true);
        assert!(pattern.is_dir_only == true);
        assert!(pattern.is_anchored == true);
    }

    #[test]
    fn recursive_glob() {
        let pattern = Pattern::parse("**/foo").unwrap();
        assert_eq!(pattern.tokens.len(), 3);
        let expected = vec![
            Token::AnyPath,
            Token::Slash,
            Token::Literal("foo".to_string()),
        ];
        assert_eq!(pattern.tokens, expected);
        assert!(pattern.is_negated == false);
        assert!(pattern.is_dir_only == false);
        assert!(pattern.is_anchored == true);
    }

    #[test]
    fn char_class() {
        let pattern = Pattern::parse("file[0-9].txt").unwrap();
        assert_eq!(pattern.tokens.len(), 3);
        let expected = vec![
            Token::Literal("file".to_string()),
            Token::CharClass {
                chars: vec!['0', '-', '9'],
                negated: false,
            },
            Token::Literal(".txt".to_string()),
        ];
        assert_eq!(pattern.tokens, expected);
        assert!(pattern.is_negated == false);
        assert!(pattern.is_dir_only == false);
        assert!(pattern.is_anchored == false);
    }

    #[test]
    fn negated_char_class() {
        let pattern = Pattern::parse("file[!0-9].txt").unwrap();
        assert_eq!(pattern.tokens.len(), 3);
        let expected = vec![
            Token::Literal("file".to_string()),
            Token::CharClass {
                chars: vec!['0', '-', '9'],
                negated: true,
            },
            Token::Literal(".txt".to_string()),
        ];
        assert_eq!(pattern.tokens, expected);
        assert!(pattern.is_negated == false);
        assert!(pattern.is_dir_only == false);
        assert!(pattern.is_anchored == false);
    }

    #[test]
    fn parse_empty_and_comment() {
        assert!(Pattern::parse("").is_none());
        assert!(Pattern::parse("   ").is_none());
        assert!(Pattern::parse("# This is a comment").is_none());
    }

    #[test]
    fn parse_combination() {
        let pattern = Pattern::parse("!foo/bar/**/file[0-9]-blah*.log").unwrap();
        assert_eq!(pattern.tokens.len(), 11);
        let expected = vec![
            Token::Literal("foo".to_string()),
            Token::Slash,
            Token::Literal("bar".to_string()),
            Token::Slash,
            Token::AnyPath,
            Token::Slash,
            Token::Literal("file".to_string()),
            Token::CharClass {
                chars: vec!['0', '-', '9'],
                negated: false,
            },
            Token::Literal("-blah".to_string()),
            Token::AnyFile,
            Token::Literal(".log".to_string()),
        ];
        assert_eq!(pattern.tokens, expected);
        assert!(pattern.is_negated == true);
        assert!(pattern.is_dir_only == false);
        assert!(pattern.is_anchored == true);
    }

    #[test]
    fn match_simple_asterisk() {
        let pattern = Pattern::parse("*.log").unwrap();
        assert!(pattern.matches("error.log", false));
        assert!(!pattern.matches("error.txt", false));
        assert!(pattern.matches("foo/bar/error.log", false));
    }

    #[test]
    fn match_anchored() {
        let pattern = Pattern::parse("/target").unwrap();
        assert!(pattern.matches("target", true));
        assert!(!pattern.matches("foo/target", true));
    }

    #[test]
    fn dir_wildcard() {
        let pattern = Pattern::parse("foo/*/bar").unwrap();
        assert!(pattern.matches("foo/baz/bar", false));
        assert!(pattern.matches("foo/test/bar", false));
        assert!(!pattern.matches("foo/baz/qux/bar", false));
        assert!(!pattern.matches("foo/bar", false));
    }

    #[test]
    fn recursive_wildcard() {
        let pattern = Pattern::parse("foo/**/bar").unwrap();
        assert!(pattern.matches("foo/baz/bar", false));
        assert!(pattern.matches("foo/test/bar", false));
        assert!(pattern.matches("foo/baz/qux/bar", false));
        assert!(pattern.matches("foo/bar", false));
        assert!(!pattern.matches("foo/baz/qux/bar/baz", false));
        assert!(!pattern.matches("foobarbaz", false));
    }

    #[test]
    fn single_char_wildcard() {
        let pattern = Pattern::parse("file?.txt").unwrap();
        assert!(pattern.matches("file1.txt", false));
        assert!(pattern.matches("fileA.txt", false));
        assert!(!pattern.matches("file12.txt", false));
        assert!(!pattern.matches("file.txt", false));
    }

    #[test]
    fn ignore_matches() {
        let ignore = Ignore {
            patterns: ["*.log", "!important.log"]
                .iter()
                .filter_map(|l| Pattern::parse(l))
                .collect(),
        };

        assert!(ignore.is_file_ignored("error.log", false));
        assert!(!ignore.is_file_ignored("important.log", false)); // negation wins
        assert!(!ignore.is_file_ignored("main.rs", false));
    }
}
