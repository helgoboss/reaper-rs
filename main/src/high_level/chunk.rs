use std::rc::Rc;

type ChunkString = String;
type ChunkStr = str;

#[derive(Clone)]
pub struct Chunk {
    content: Rc<ChunkString>
}

#[derive(Clone)]
pub struct ChunkRegion {
    parent_chunk: Chunk,
    start_pos: usize,
    length: usize,
}

impl Chunk {
    pub fn new(content: ChunkString) -> Chunk {
        Chunk {
            content: Rc::new(content)
        }
    }

    pub fn get_content(&self) -> Rc<ChunkString> {
        self.content.clone()
    }

    pub fn get_region(&self) -> ChunkRegion {
        ChunkRegion::new(self.clone(), 0, self.content.len())
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

    pub fn find_line_starting_with(&self, needle: &ChunkStr) -> Option<ChunkRegion> {
        if !self.is_valid() {
            return None;
        }
        self.find_first_string_at_line_start(needle).map(|needle_region| {
            needle_region.move_right_cursor_right_to_end_of_current_line()
        })
    }

    pub fn is_valid(&self) -> bool {
        self.length != usize::max_value() &&
            self.start_pos + self.length <= self.parent_chunk.content.len()
    }

    pub fn find_first_string_at_line_start(&self, needle: &ChunkStr) -> Option<ChunkRegion> {
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
        let pattern = ChunkString::from("\n") + needle;
        content.find(&pattern).map(|rel_pos| {
            ChunkRegion::new(self.parent_chunk.clone(), self.start_pos + rel_pos + 1, needle.len())
        })
    }

    pub fn get_content(&self) -> &ChunkStr {
        let full_string_ref = &self.parent_chunk.content[..];
        if self.is_valid() {
            &full_string_ref[self.start_pos..(self.start_pos + self.length)]
        } else {
            ""
        }
    }

    pub fn move_right_cursor_right_to_end_of_current_line(&self) -> ChunkRegion {
        self.move_right_cursor_right_to_start_of("\n")
    }

    pub fn move_right_cursor_right_to_start_of(&self, needle: &ChunkStr) -> ChunkRegion {
        if !self.is_valid() {
            return self.clone();
        }
        let after = self.get_after();
        if !after.is_valid() {
            return self.clone();
        }
        match after.get_content().find(needle) {
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
            self.parent_chunk.content.len() - self.get_end_pos_plus_one()
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