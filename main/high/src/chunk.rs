use reaper_medium::ReaperString;
use std::cell::{Ref, RefCell};
use std::convert::TryFrom;

use std::fmt::{Display, Formatter};
use std::rc::Rc;

// Cheap to clone because string is shared
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chunk {
    content: Rc<RefCell<String>>,
}

impl From<ReaperString> for Chunk {
    fn from(value: ReaperString) -> Self {
        Chunk::new(value.into_string())
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content.borrow().as_str())
    }
}

impl TryFrom<Chunk> for String {
    type Error = &'static str;

    fn try_from(value: Chunk) -> Result<Self, Self::Error> {
        let freed_prisoner = Rc::try_unwrap(value.content)
            .map_err(|_| "Can't convert Chunk to CString when there are still other references")?;
        Ok(freed_prisoner.into_inner())
    }
}

// Cheap to clone. Owns chunk for ease of use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkRegion {
    parent_chunk: Chunk,
    start_pos: usize,
    length: usize,
}

impl Display for ChunkRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content())
    }
}

impl Chunk {
    pub fn new(content: String) -> Chunk {
        Chunk {
            content: Rc::new(RefCell::new(content)),
        }
    }

    pub fn content(&self) -> Rc<RefCell<String>> {
        self.content.clone()
    }

    pub fn region(&self) -> ChunkRegion {
        ChunkRegion::new(self.clone(), 0, self.content.borrow().len())
    }

    pub fn insert_before_region(&mut self, region: &ChunkRegion, content: &str) {
        self.require_valid_region(region);
        self.content
            .borrow_mut()
            .insert_str(region.start_pos(), content);
    }

    pub fn insert_after_region(&mut self, region: &ChunkRegion, content: &str) {
        self.require_valid_region(region);
        self.content
            .borrow_mut()
            .insert_str(region.end_pos_plus_one(), content);
    }

    pub fn insert_before_region_as_block(&mut self, region: &ChunkRegion, content: &str) {
        self.insert_before_region(region, content);
        self.insert_new_lines_if_necessary_at(
            region.start_pos(),
            region.start_pos() + content.len(),
        );
    }

    pub fn insert_after_region_as_block(&mut self, region: &ChunkRegion, content: &str) {
        self.insert_after_region(region, content);
        self.insert_new_lines_if_necessary_at(
            region.end_pos_plus_one(),
            region.end_pos_plus_one() + content.len(),
        );
    }

    pub fn insert_new_lines_if_necessary_at(&mut self, pos1: usize, pos2: usize) {
        let inserted = self.insert_new_line_if_necessary_at(pos1);
        self.insert_new_line_if_necessary_at(if inserted { pos2 + 1 } else { pos2 });
    }

    pub fn insert_new_line_if_necessary_at(&mut self, pos: usize) -> bool {
        if pos == 0 {
            return false;
        }
        let mut content = self.content.borrow_mut();
        if pos >= content.len() - 1 {
            return false;
        }
        if content[pos..].starts_with('\n') {
            return false;
        }
        content.insert(pos, '\n');
        true
    }

    pub fn delete_region(&mut self, region: &ChunkRegion) {
        self.require_valid_region(region);
        self.content
            .borrow_mut()
            .replace_range(region.start_pos..(region.start_pos + region.length), "");
    }

    pub fn replace_region(&mut self, region: &ChunkRegion, content: &str) {
        self.require_valid_region(region);
        self.content.borrow_mut().replace_range(
            region.start_pos..(region.start_pos + region.length),
            content,
        );
    }

    fn require_valid_region(&self, region: &ChunkRegion) {
        if !region.is_valid() {
            panic!("Invalid chunk region")
        }
    }
}

impl ChunkRegion {
    fn new(parent_chunk: Chunk, start_pos: usize, length: usize) -> ChunkRegion {
        ChunkRegion {
            parent_chunk,
            start_pos,
            length,
        }
    }

    pub fn parent_chunk(&self) -> Chunk {
        self.parent_chunk.clone()
    }

    pub fn first_line(&self) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        self.content()
            .find('\n')
            .map(|rel_pos| self.create_region_from_relative_start_pos(0, rel_pos))
            .unwrap_or_else(|| self.clone())
    }

    pub fn last_line(&self) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        self.content()
            .rfind('\n')
            .map(|rel_pos_of_newline| {
                let rel_pos_after_newline = rel_pos_of_newline + 1;
                self.create_region_from_relative_start_pos(
                    rel_pos_after_newline,
                    self.length() - rel_pos_after_newline,
                )
            })
            .unwrap_or_else(|| self.clone())
    }

    pub fn find_line_starting_with(&self, needle: &str) -> Option<ChunkRegion> {
        if !self.is_valid() {
            return None;
        }
        self.find_first_string_at_line_start(needle)
            .map(|needle_region| needle_region.move_right_cursor_right_to_end_of_current_line())
    }

    pub fn is_valid(&self) -> bool {
        self.length != usize::MAX
            && self.start_pos + self.length <= self.parent_chunk.content.borrow().len()
    }

    pub fn find_first_string_at_line_start(&self, needle: &str) -> Option<ChunkRegion> {
        if !self.is_valid() {
            return None;
        }
        let content = self.content();
        if content.len() < needle.len() {
            return None;
        }
        if &content[0..needle.len()] == needle {
            return Some(ChunkRegion::new(self.parent_chunk.clone(), 0, needle.len()));
        }
        let pattern = String::from("\n") + needle;
        content.find(&pattern).map(|rel_pos| {
            ChunkRegion::new(
                self.parent_chunk.clone(),
                self.start_pos + rel_pos + 1,
                needle.len(),
            )
        })
    }

    pub fn content(&self) -> Ref<str> {
        if self.is_valid() {
            Ref::map(self.parent_chunk.content.borrow(), |r| {
                &r[self.start_pos..(self.start_pos + self.length)]
            })
        } else {
            // For now I don't know how to simply create a Ref<"">
            #[allow(clippy::reversed_empty_ranges)]
            Ref::map(self.parent_chunk.content.borrow(), |r| &r[0..0])
        }
    }

    pub fn move_right_cursor_right_to_end_of_current_line(&self) -> ChunkRegion {
        self.move_right_cursor_right_to_start_of("\n")
    }

    pub fn move_left_cursor_right_to_start_of_next_line(&self) -> ChunkRegion {
        self.move_left_cursor_right_to_start_of_line_beginning_with("")
    }

    pub fn move_left_cursor_right_to_start_of_line_beginning_with(
        &self,
        needle: &str,
    ) -> ChunkRegion {
        self.move_left_cursor_right_to_start_of((String::from("\n") + needle).as_str())
            .move_left_cursor_right_by(1)
    }

    pub fn move_left_cursor_right_to_start_of(&self, needle: &str) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        let rel_pos = match self.content().find(needle) {
            None => return self.create_invalid_region(),
            Some(p) => p,
        };
        self.move_left_cursor_right_by(rel_pos)
    }

    pub fn move_right_cursor_left_to_start_of(&self, needle: &str) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        let rel_start_pos_of_needle = match self.content().rfind(needle) {
            None => return self.create_invalid_region(),
            Some(p) => p,
        };
        self.create_region_from_relative_start_pos(0, rel_start_pos_of_needle)
    }

    pub fn move_right_cursor_left_to_end_of_previous_line(&self) -> ChunkRegion {
        self.move_right_cursor_left_to_start_of("\n")
    }

    pub fn move_right_cursor_right_to_start_of(&self, needle: &str) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        let after = self.after();
        if !after.is_valid() {
            return self.clone();
        }
        let after_content = after.content();
        match after_content.find(needle) {
            None => self.create_invalid_region(),
            Some(rel_pos) => self.move_right_cursor_right_by(rel_pos),
        }
    }

    pub fn move_left_cursor_left_to_start_of_line_beginning_with(
        &self,
        needle: &str,
    ) -> ChunkRegion {
        self.move_left_cursor_left_to_start_of((String::from("\n") + needle).as_str())
            .move_left_cursor_right_by(1)
    }

    pub fn move_left_cursor_right_by(&self, count: usize) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        self.move_left_cursor_to(self.start_pos + count)
    }

    pub fn move_left_cursor_to(&self, start_pos: usize) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        let new_length = self.length + self.start_pos - start_pos;
        ChunkRegion::new(self.parent_chunk.clone(), start_pos, new_length)
    }

    pub fn move_left_cursor_left_to_start_of(&self, needle: &str) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        let before = self.before();
        if !before.is_valid() {
            return self.create_invalid_region();
        }
        let start_pos_of_needle = match before.content().rfind(needle) {
            None => return self.create_invalid_region(),
            Some(p) => p,
        };
        self.move_left_cursor_to(start_pos_of_needle)
    }

    pub fn move_right_cursor_right_to_start_of_line_beginning_with(
        &self,
        needle: &str,
    ) -> ChunkRegion {
        self.move_right_cursor_right_to_start_of((String::from("\n") + needle).as_str())
            .move_right_cursor_right_by(1)
    }

    pub fn before(&self) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        ChunkRegion::new(self.parent_chunk.clone(), 0, self.start_pos)
    }

    pub fn after(&self) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        ChunkRegion::new(
            self.parent_chunk.clone(),
            self.end_pos_plus_one(),
            self.parent_chunk.content.borrow().len() - self.end_pos_plus_one(),
        )
    }

    pub fn end_pos_plus_one(&self) -> usize {
        self.start_pos + self.length
    }

    pub fn start_pos(&self) -> usize {
        self.start_pos
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn move_right_cursor_right_by(&self, count: usize) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        self.create_region_from_relative_start_pos(0, self.length + count)
    }

    // Returns the tag completely from < to >
    pub fn find_first_tag_named(
        &self,
        relative_search_start_pos: usize,
        tag_name: &str,
    ) -> Option<ChunkRegion> {
        if !self.is_valid() {
            return None;
        }
        let tag_opener_with_new_line = format!("\n<{tag_name}");
        self.find_followed_by_one_of(&tag_opener_with_new_line, " \n", relative_search_start_pos)
            .and_then(|pos| self.parse_tag_starting_from(pos + 1))
    }

    // Returns the tag completely from < to >
    // TODO-low Why don't we return an invalid chunk region instead of none? That would allow easier
    // chaining and would  be more in line with the other methods.
    pub fn find_first_tag(&self, relative_search_start_pos: usize) -> Option<ChunkRegion> {
        if !self.is_valid() {
            return None;
        }
        let content = self.content();
        if content[relative_search_start_pos..].starts_with('<') {
            self.parse_tag_starting_from(relative_search_start_pos)
        } else {
            let tag_opener_with_new_line = "\n<";
            content[relative_search_start_pos..]
                .find(tag_opener_with_new_line)
                .and_then(|super_relative_tag_opener_with_new_line_pos| {
                    let rel_tag_opener_with_new_line_pos =
                        relative_search_start_pos + super_relative_tag_opener_with_new_line_pos;
                    self.parse_tag_starting_from(rel_tag_opener_with_new_line_pos + 1)
                })
        }
    }

    pub fn starts_with(&self, needle: &str) -> bool {
        if !self.is_valid() {
            return false;
        }
        self.content().starts_with(needle)
    }

    pub fn ends_with(&self, needle: &str) -> bool {
        if !self.is_valid() {
            return false;
        }
        self.content().ends_with(needle)
    }

    pub fn contains(&self, needle: &str) -> bool {
        if !self.is_valid() {
            return false;
        }
        self.content().contains(needle)
    }

    // Precondition: isValid
    fn find_followed_by_one_of(
        &self,
        needle: &str,
        one_of: &str,
        mut rel_start_pos: usize,
    ) -> Option<usize> {
        let content = self.content();
        while rel_start_pos < content.len() {
            let needle_pos_relative_to_rel_start_pos = match content[rel_start_pos..].find(needle) {
                None => return None, // Needle not found
                Some(p) => p,
            };
            // Needle found
            let rel_needle_pos = rel_start_pos + needle_pos_relative_to_rel_start_pos;
            let rel_following_char_pos = rel_needle_pos + needle.len();
            if rel_following_char_pos < content.len() {
                // String goes on after needle
                let following_char = content[rel_following_char_pos..].chars().next().unwrap();
                if one_of.find(following_char).is_some() {
                    // Found complete match
                    return Some(rel_needle_pos);
                } else {
                    // No complete match yet. Go on searching.
                    rel_start_pos = rel_following_char_pos + 1;
                }
            } else {
                // No complete match found
                return None;
            }
        }
        // No complete match found
        None
    }

    // Precondition: isValid
    fn parse_tag_starting_from(&self, rel_tag_opener_pos: usize) -> Option<ChunkRegion> {
        let mut rel_start_pos = rel_tag_opener_pos + 1;
        let mut open_levels_count = 1;
        let content = self.content();
        while rel_start_pos < content.len() {
            let rel_tag_opener_or_closer_pos =
                match self.find_followed_by_one_of("\n", "<>", rel_start_pos) {
                    None => return None, // No further tag opener or closer found
                    Some(p) => p,
                };
            // Further tag opener or closer found
            let rel_tag_opener_or_closer_without_newline_pos = rel_tag_opener_or_closer_pos + 1;
            let tag_opener_or_closer_without_newline = content
                [rel_tag_opener_or_closer_without_newline_pos..]
                .chars()
                .next()
                .unwrap();
            if tag_opener_or_closer_without_newline == '<' {
                // Opening tag (nested)
                open_levels_count += 1;
                rel_start_pos = rel_tag_opener_or_closer_without_newline_pos + 1;
            } else {
                // Closing tag
                open_levels_count -= 1;
                if open_levels_count == 0 {
                    // Found tag closer of searched tag
                    let length =
                        rel_tag_opener_or_closer_without_newline_pos - rel_tag_opener_pos + 1;
                    return Some(
                        self.create_region_from_relative_start_pos(rel_tag_opener_pos, length),
                    );
                } else {
                    // Nested tag was closed
                    rel_start_pos = rel_tag_opener_or_closer_without_newline_pos + 1;
                }
            }
        }
        // Tag closer not found
        None
    }

    fn create_invalid_region(&self) -> ChunkRegion {
        ChunkRegion::new(self.parent_chunk.clone(), self.start_pos, usize::MAX)
    }

    fn create_region_from_relative_start_pos(
        &self,
        rel_start_pos: usize,
        length: usize,
    ) -> ChunkRegion {
        ChunkRegion::new(
            self.parent_chunk.clone(),
            self.start_pos + rel_start_pos,
            length,
        )
    }
}
