use crate::Entry;

pub trait Journal {
    fn get_past_entries() -> Vec<Entry>;
    fn get_today_entry() -> Option<Entry>;
    fn store_new_entry();
}

struct SimpleSqliteJournal {
    path: String
}

impl Journal for SimpleSqliteJournal {
    fn get_past_entries() -> Vec<Entry> {
        todo!()
    }

    fn get_today_entry() -> Option<Entry> {
        todo!()
    }

    fn store_new_entry() {
        todo!()
    }
}