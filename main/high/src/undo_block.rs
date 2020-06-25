use crate::{Project, Reaper};
use reaper_medium::ReaperStr;

// Constructor takes care of starting the undo block. Destructor takes care of ending the undo block
// (RAII).
pub(super) struct UndoBlock<'a> {
    label: &'a ReaperStr,
    project: Project,
}

impl UndoBlock<'_> {
    pub(crate) fn new(project: Project, label: &ReaperStr) -> UndoBlock {
        UndoBlock { label, project }
    }
}

impl Drop for UndoBlock<'_> {
    fn drop(&mut self) {
        Reaper::get().leave_undo_block_internal(self.project, self.label);
    }
}
