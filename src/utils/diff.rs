use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

#[derive(Debug, Clone)]
pub enum Edit<'a> {
    Keep(&'a str),
    Delete(&'a str),
    Insert(&'a str),
}

pub fn myers_diff<'a>(old: &'a str, new: &'a str) -> Vec<Edit<'a>> {
    let old_lines = old.lines().collect::<Vec<_>>();
    let new_lines = new.lines().collect::<Vec<_>>();

    let mut memo = HashMap::new();
    let diff = myers_rec(&old_lines, &new_lines, 0, 0, &mut memo);

    // Explicitly drop the memo to ensure we have the only reference to the edits in the Rc
    drop(memo);
    // At this point we should have the only reference to the edits in the Rc
    Rc::try_unwrap(diff)
        .map_err(|_| anyhow::anyhow!("Unexpected shared reference in diff result"))
        .unwrap()
        .into()
}

// We use VecDeque here for O(1) prepend. A Vec would be O(n) shifting every element "to the right"
// TODO: Eventually we should rewrite this non recursively but until then Rc makes the memo clones really cheap.
fn myers_rec<'a>(
    old: &[&'a str],
    new: &[&'a str],
    old_idx: usize,
    new_idx: usize,
    memo: &mut HashMap<(usize, usize), Rc<VecDeque<Edit<'a>>>>,
) -> Rc<VecDeque<Edit<'a>>> {
    if let Some(cached) = memo.get(&(old_idx, new_idx)) {
        return Rc::clone(cached);
    }

    let result = match (old.get(old_idx), new.get(new_idx)) {
        // Both exhausted, no edits needed
        (None, None) => VecDeque::new(),
        // Old exhausted, insert remaining new lines
        (None, Some(&line)) => {
            let mut rest = myers_rec(old, new, old_idx, new_idx + 1, memo)
                .as_ref()
                .clone();
            rest.push_front(Edit::Insert(line));
            rest
        }
        // New exhausted, delete remaining old lines
        (Some(&line), None) => {
            let mut rest = myers_rec(old, new, old_idx + 1, new_idx, memo)
                .as_ref()
                .clone();
            rest.push_front(Edit::Delete(line));
            rest
        }
        // Lines match, keep and advance both
        (Some(&old_line), Some(&new_line)) if old_line == new_line => {
            let mut rest = myers_rec(old, new, old_idx + 1, new_idx + 1, memo)
                .as_ref()
                .clone();
            rest.push_front(Edit::Keep(old_line));
            rest
        }
        // Lines differ, pick whichever of delete/insert leads to fewer edits
        (Some(&old_line), Some(&new_line)) => {
            let delete = myers_rec(old, new, old_idx + 1, new_idx, memo);
            let insert = myers_rec(old, new, old_idx, new_idx + 1, memo);
            if delete.len() <= insert.len() {
                let mut rest = delete.as_ref().clone();
                rest.push_front(Edit::Delete(old_line));
                rest
            } else {
                let mut rest = insert.as_ref().clone();
                rest.push_front(Edit::Insert(new_line));
                rest
            }
        }
    };

    let result = Rc::new(result);
    memo.insert((old_idx, new_idx), Rc::clone(&result));
    result
}

#[derive(Debug)]
pub struct Hunk<'a> {
    pub old_start: usize,
    pub new_start: usize,
    pub old_count: usize,
    pub new_count: usize,
    pub edits: Vec<&'a Edit<'a>>,
}

pub fn compute_hunks<'a>(edits: &'a [Edit<'a>]) -> Vec<Hunk<'a>> {
    let mut hunks = Vec::new();
    let mut old_line = 1;
    let mut new_line = 1;
    let mut current: Option<Hunk<'a>> = None;

    for (i, edit) in edits.iter().enumerate() {
        let start = (i as isize - 3).max(0) as usize;
        let end = (i + 3).min(edits.len() - 1);
        let near_change = edits[start..=end]
            .iter()
            .any(|e| !matches!(e, Edit::Keep(_)));

        if near_change {
            let hunk = current.get_or_insert(Hunk {
                old_start: old_line,
                new_start: new_line,
                old_count: 0,
                new_count: 0,
                edits: Vec::new(),
            });
            hunk.edits.push(edit);
            match edit {
                Edit::Keep(_) => {
                    hunk.old_count += 1;
                    hunk.new_count += 1;
                }
                Edit::Delete(_) => hunk.old_count += 1,
                Edit::Insert(_) => hunk.new_count += 1,
            }
        } else if let Some(hunk) = current.take() {
            hunks.push(hunk);
        }

        match edit {
            Edit::Keep(_) => {
                old_line += 1;
                new_line += 1;
            }
            Edit::Delete(_) => old_line += 1,
            Edit::Insert(_) => new_line += 1,
        }
    }

    if let Some(hunk) = current {
        hunks.push(hunk);
    }

    hunks
}
