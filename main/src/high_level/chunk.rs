use std::rc::Rc;
use std::cell::{RefCell, Ref};
use std::ffi::CString;

// Cheap to clone because string is shared
#[derive(Clone)]
pub struct Chunk {
    content: Rc<RefCell<String>>
}

impl From<CString> for Chunk {
    fn from(value: CString) -> Self {
        Chunk::new(value.into_string().expect("Chunk content contains illegal characters"))
    }
}

impl From<Chunk> for CString {
    fn from(value: Chunk) -> Self {
        // TODO Is this too expensive? Check if we can rely on the chunk to be okay and use an
        //  unsafe conversion.
        let freed_prisoner = Rc::try_unwrap(value.content)
            .expect("Can't convert Chunk to CString when there are still other references");
        CString::new(freed_prisoner.into_inner()).expect("Chunk contained 0-bytes")
    }
}

// Cheap to clone. Owns chunk for ease of use.
#[derive(Clone)]
pub struct ChunkRegion {
    parent_chunk: Chunk,
    start_pos: usize,
    length: usize,
}

impl Chunk {
    pub fn new(content: String) -> Chunk {
        Chunk {
            content: Rc::new(RefCell::new(content))
        }
    }

    pub fn get_content(&self) -> Rc<RefCell<String>> {
        self.content.clone()
    }

    pub fn get_region(&self) -> ChunkRegion {
        ChunkRegion::new(self.clone(), 0, self.content.borrow().len())
    }

    pub fn insert_after_region(&mut self, region: &ChunkRegion, content: &str) {
        self.require_valid_region(region);
        self.content.borrow_mut().insert_str(region.get_end_pos_plus_one(), content);
    }

    pub fn insert_after_region_as_block(&mut self, region: &ChunkRegion, content: &str) {
        self.insert_after_region(region, content);
        self.insert_new_lines_if_necessary_at(region.get_end_pos_plus_one(), region.get_end_pos_plus_one() + content.len());
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
        if content[pos..].chars().next() == Some('\n') {
            return false
        }
        content.insert(pos, '\n');
        true
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

    pub fn get_first_line(&self) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        self.get_content().find('\n').map(|rel_pos| {
            self.create_region_from_relative_start_pos(0, rel_pos)
        }).unwrap_or_else(|| self.clone())
    }

    pub fn find_line_starting_with(&self, needle: &str) -> Option<ChunkRegion> {
        if !self.is_valid() {
            return None;
        }
        self.find_first_string_at_line_start(needle).map(|needle_region| {
            needle_region.move_right_cursor_right_to_end_of_current_line()
        })
    }

    pub fn is_valid(&self) -> bool {
        self.length != usize::max_value() &&
            self.start_pos + self.length <= self.parent_chunk.content.borrow().len()
    }

    pub fn find_first_string_at_line_start(&self, needle: &str) -> Option<ChunkRegion> {
        if !self.is_valid() {
            return None;
        }
        let content = self.get_content();
        if content.len() < needle.len() {
            return None;
        }
        if &content[0..needle.len()] == needle {
            return Some(ChunkRegion::new(self.parent_chunk.clone(), 0, needle.len()));
        }
        let pattern = String::from("\n") + needle;
        content.find(&pattern).map(|rel_pos| {
            ChunkRegion::new(self.parent_chunk.clone(), self.start_pos + rel_pos + 1, needle.len())
        })
    }

    pub fn get_content(&self) -> Ref<str> {
        if self.is_valid() {
            Ref::map(self.parent_chunk.content.borrow(), |r| {
                &r[self.start_pos..(self.start_pos + self.length)]
            })
        } else {
            Ref::map(self.parent_chunk.content.borrow(), |r| {
                &r[0..0]
            })
        }
    }

    pub fn move_right_cursor_right_to_end_of_current_line(&self) -> ChunkRegion {
        self.move_right_cursor_right_to_start_of("\n")
    }

    pub fn move_right_cursor_right_to_start_of(&self, needle: &str) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        let after = self.get_after();
        if !after.is_valid() {
            return self.clone();
        }
        let after_content = after.get_content();
        match after_content.find(needle) {
            None => self.create_invalid_region(),
            Some(rel_pos) => self.move_right_cursor_right_by(rel_pos)
        }
    }

    pub fn get_after(&self) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        ChunkRegion::new(
            self.parent_chunk.clone(),
            self.get_end_pos_plus_one(),
            self.parent_chunk.content.borrow().len() - self.get_end_pos_plus_one(),
        )
    }

    pub fn get_end_pos_plus_one(&self) -> usize {
        self.start_pos + self.length
    }

    pub fn get_start_pos(&self) -> usize {
        self.start_pos
    }

    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn move_right_cursor_right_by(&self, count: usize) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        self.create_region_from_relative_start_pos(0, self.length + count)
    }

    fn create_invalid_region(&self) -> ChunkRegion {
        ChunkRegion::new(self.parent_chunk.clone(), self.start_pos, usize::max_value())
    }

    fn create_region_from_relative_start_pos(&self, rel_start_pos: usize, length: usize) -> ChunkRegion {
        ChunkRegion::new(self.parent_chunk.clone(), self.start_pos + rel_start_pos, length)
    }
}