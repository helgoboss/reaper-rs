use splitty::SplitUnquotedChar;
use std::fmt::{Debug, Formatter};
use std::io::BufRead;

/// This is a streaming pull parser.
///
/// Pros:
///
/// - Very low memory requirements. At any point in time, only one line needs to be kept in memory,
///   not the complete chunk.
///
/// Cons:
///
/// - This doesn't implement the convenient `Iterator` trait, simply because it's not possible.
///   It lends data from a mutable buffer that would turn invalid on the next `next` call.
///   Maybe it's possible one day when Rust supports lending/streaming iterators.
/// - Although memory usage is low, it needs a buffer that must be at least a big as the
///   longest line in the chunk. If you choose the initial capacity too small, you can expect
///   a few allocations until the parser ends up with the maximum size.
///
/// Verdict: Only use this if you want to keep memory usage during parsing to an absolute minimum!
pub struct StreamingParser<S> {
    source: S,
    buffer: String,
}

impl<S> StreamingParser<S> {
    /// Creates the streaming parser.
    pub fn new(source: S, initial_capacity: usize) -> Self {
        Self {
            source,
            buffer: String::with_capacity(initial_capacity),
        }
    }
}

#[allow(clippy::should_implement_trait)]
impl<S: BufRead> StreamingParser<S> {
    /// Repeatedly call `next` to read the events.
    pub fn next(&mut self) -> Option<Item> {
        self.buffer.clear();
        if self.source.read_line(&mut self.buffer).ok()? == 0 {
            return None;
        }
        Some(Item::parse_from_line(&self.buffer))
    }
}

/// This is a non-streaming pull parser.
///
/// Pros:
///
/// - Exposes a standard iterator, very convenient.
/// - Never allocates.
///
/// Cons:
///
/// - Requires the complete chunk to reside in memory.
///
/// Verdict: Use this unless you want to keep memory usage during parsing to an absolute minimum!
pub struct OneShotParser<'a> {
    source: &'a str,
}

impl<'a> OneShotParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn events(&self) -> impl Iterator<Item = Event> {
        self.source.lines().map(|l| {
            let (start, end) = get_range(self.source, l);
            Event::new(self.source, start, end, Item::parse_from_line(l))
        })
    }
}

#[derive(Debug)]
pub struct Event<'a> {
    pub rppxml: &'a str,
    pub start: usize,
    pub end: usize,
    pub item: Item<'a>,
}

impl<'a> Event<'a> {
    pub fn new(rppxml: &'a str, start: usize, end: usize, item: Item<'a>) -> Self {
        Self {
            rppxml,
            start,
            end,
            item,
        }
    }

    pub fn line(&self) -> &'a str {
        &self.rppxml[self.start..self.end]
    }
}

#[derive(Debug)]
pub enum Item<'a> {
    StartTag(Element<'a>),
    EndTag,
    Attribute(Element<'a>),
    Content(&'a str),
    Empty,
}

impl<'a> Item<'a> {
    fn parse_from_line(line: &'a str) -> Self {
        let line = line.trim();
        if let Some(remainder) = line.strip_prefix('<') {
            if let Some(el) = Element::parse(remainder) {
                Item::StartTag(el)
            } else {
                Item::Content(line)
            }
        } else if line.starts_with('>') {
            Item::EndTag
        } else if line.is_empty() {
            Item::Empty
        } else if let Some(el) = Element::parse(line) {
            Item::Attribute(el)
        } else {
            Item::Content(line)
        }
    }
}

fn get_range(whole_buffer: &str, part: &str) -> (usize, usize) {
    let start = part.as_ptr() as usize - whole_buffer.as_ptr() as usize;
    let end = start + part.len();
    (start, end)
}

pub struct Element<'a>(&'a str, SplitUnquotedChar<'a>);

impl<'a> Element<'a> {
    pub fn name(&self) -> &'a str {
        self.0
    }

    pub fn into_values(self) -> SplitUnquotedChar<'a> {
        self.1
    }

    fn parse(remainder: &'a str) -> Option<Self> {
        let mut split = splitty::split_unquoted_whitespace(remainder).unwrap_quotes(true);
        let first_word = split.next()?;
        if !first_word
            .chars()
            .all(|c: char| c.is_ascii_uppercase() || c == '_')
        {
            return None;
        }
        Some(Element(first_word, split))
    }
}

impl<'a> Debug for Element<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Element").field(&self.0).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn one_shot_basics() {
        let text = include_str!("examples/fx-chain-tag.rpp");
        let parser = OneShotParser::new(text);
        let event = parser.events().next().unwrap();
        assert_eq!(event.start, 0);
        assert_eq!(event.end, 8);
        let Item::StartTag(Element("FXCHAIN", mut values)) = event.item else {
            panic!();
        };
        assert_eq!(values.next(), None);
    }

    #[test]
    fn streaming_basics() {
        let text = include_str!("examples/fx-chain-tag.rpp");
        let reader = BufReader::new(text.as_bytes());
        let mut events = StreamingParser::new(reader, 1000);
        let Some(Item::StartTag(Element("FXCHAIN", mut values))) = events.next() else {
            panic!();
        };
        assert_eq!(values.next(), None);
    }
}
