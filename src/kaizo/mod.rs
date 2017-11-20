mod driver;
pub use self::driver::drive;

use std::str::pattern;
// const UNICODE_ARROWS: [char; 2] = ['\u{2192}', '\u{21d2}'];

#[derive(Copy, Clone, Debug)]
struct ArrowPattern;
impl<'a> pattern::Pattern<'a> for ArrowPattern {
    type Searcher = ArrowSearcher<'a>;

    fn into_searcher(self, haystack: &'a str) -> ArrowSearcher<'a> {
        ArrowSearcher(haystack, 0)
    }
}

#[derive(Debug)]
struct ArrowSearcher<'a>(&'a str, usize);
unsafe impl<'a> pattern::Searcher<'a> for ArrowSearcher<'a> {
    fn haystack(&self) -> &'a str { self.0 }
    fn next(&mut self) -> pattern::SearchStep {
        let bytes = self.haystack().as_bytes();
        let len = bytes.len();
        let start = self.1;
        for p in start..len {
            match bytes[p] {
                b'>' => {
                    self.1 = p + 1;
                    return pattern::SearchStep::Match(start, self.1)
                },
                b'-' => {},
                _ => {
                    self.1 = p + 1;
                    return pattern::SearchStep::Reject(start, self.1)
                },
            }
        }
        self.1 = len;
        if start == len {
            pattern::SearchStep::Done
        } else {
            pattern::SearchStep::Reject(start, len)
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum RequestData<'a> {
    One(&'a str),
    Two(&'a str, &'a str),
    Empty,
}

fn split_with_arrow(s: &str) -> RequestData {
    let args = s.split(ArrowPattern).collect::<Vec<_>>();
    if args.len() == 2 && args[0].trim().len() > 0 && args[1].trim().len() > 0 {
        RequestData::Two(args[0].trim(), args[1].trim())
    } else {
        let s = s.trim();
        if s.len() > 0 {
            RequestData::One(s)
        } else {
            RequestData::Empty
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct Kaizo {
    pub screen_name: String,
    pub status_id: u64,
    pub from: String,
    pub to: String,
}

impl Kaizo {
    pub fn new<S>(screen_name: S, status_id: u64, command: &str) -> Option<Self>
        where S: Into<String>
    {
        let command = command.replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&");
        let data = split_with_arrow(&command);
        let (from, to) = match data {
            RequestData::One(s) => (s.to_owned(), "시공".to_owned()),
            RequestData::Two(from, to) => (from.to_owned(), to.to_owned()),
            RequestData::Empty => return None,
        };
        Some(Kaizo {
            screen_name: screen_name.into(),
            status_id,
            from,
            to,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        let given = "a -> b ---> c > d";
        let expect: &[_] = &["a ", " b ", " c ", " d"];
        let result = given.split(ArrowPattern).collect::<Vec<_>>();
        assert_eq!(expect, result.as_slice());
    }

    #[test]
    fn test_parse_data() {
        let given = "내 이름은 메구밍!";
        let result = split_with_arrow(given);
        match result {
            RequestData::One(s) => assert_eq!(s, given),
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_trim() {
        let given = "  아크 위저드를 생업으로    ";
        let expect = "아크 위저드를 생업으로";
        let result = split_with_arrow(given);
        match result {
            RequestData::One(s) => assert_eq!(s, expect),
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_arrow() {
        let given = "삼고 --> 있으며,";
        let expect_from = "삼고";
        let expect_to = "있으며,";
        let result = split_with_arrow(given);
        match result {
            RequestData::Two(a, b) => {
                assert_eq!(a, expect_from);
                assert_eq!(b, expect_to);
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_multiple_arrow() {
        let given = "최강의 > 공격마법, -> 폭렬마법을 다루는 자!";
        let result = split_with_arrow(given);
        match result {
            RequestData::One(s) => assert_eq!(s, given),
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_empty() {
        let given = "";
        let result = split_with_arrow(given);
        match result {
            RequestData::Empty => {},
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_empty_whitespace() {
        let given = "     ";
        let result = split_with_arrow(given);
        match result {
            RequestData::Empty => {},
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_only_arrow() {
        let given = "->";
        let result = split_with_arrow(given);
        match result {
            RequestData::One(s) => assert_eq!(s, given),
            _ => panic!(),
        }
    }
}
