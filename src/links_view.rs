use crate::{user::User, window::Entry};
use std::collections::BTreeMap;

pub struct LinkSort<'a>(pub &'a [LinkEntry]);

impl<'a> LinkSort<'a> {
    // NOTE: 0 is reserved
    pub fn sorted_by_ts(&self) -> Vec<(usize, &LinkEntry)> {
        let mut out: Vec<(usize, &LinkEntry)> = vec![];

        let mut pos = 0;
        let mut seen = BTreeMap::new();

        for link in self.0 {
            let pos = seen.entry((link.ts, &*link.user.name)).or_insert_with(|| {
                pos += 1;
                pos
            });

            out.push((*pos, link));
        }

        out.sort_unstable_by(|(_, left), (_, right)| left.ts.cmp(&right.ts));
        out
    }
}

#[derive(Debug, Clone)]
pub struct LinkEntry {
    pub user: User,
    pub link: String,
    pub ts: chrono::DateTime<chrono::Local>,
}

impl LinkEntry {
    pub fn new(link: impl ToString, entry: &Entry) -> Self {
        Self {
            user: entry.user.clone(),
            ts: entry.ts,
            link: link.to_string(),
        }
    }
}
