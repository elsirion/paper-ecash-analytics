mod note;
mod note_set;
mod oob_notes;
mod paper_note;

pub use note::{Note, NoteStatus, SpendInfo};
pub use note_set::NoteSet;
pub use oob_notes::{parse_oob_notes, parse_csv_notes};
pub use paper_note::{PaperNote, PaperNoteStatus, group_into_paper_notes};
