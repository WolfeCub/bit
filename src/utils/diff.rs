use std::{collections::HashMap, rc::Rc};

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
    return myers_rec(&old_lines, &new_lines, 0, 0, &mut memo).to_vec();
}

// TODO: Eventually we should rewrite this non recursively but until then Rc makes the memo clones really cheap.
fn myers_rec<'a>(
    old: &[&'a str],
    new: &[&'a str],
    old_idx: usize,
    new_idx: usize,
    memo: &mut HashMap<(usize, usize), Rc<Vec<Edit<'a>>>>,
) -> Rc<Vec<Edit<'a>>> {
    if let Some(cached) = memo.get(&(old_idx, new_idx)) {
        return Rc::clone(cached);
    }

    let result = match (old.get(old_idx), new.get(new_idx)) {
        // Both exhausted, no edits needed
        (None, None) => vec![],
        // Old exhausted, insert remaining new lines
        (None, Some(&line)) => {
            let mut rest = myers_rec(old, new, old_idx, new_idx + 1, memo)
                .as_ref()
                .clone();
            rest.insert(0, Edit::Insert(line));
            rest
        }
        // New exhausted, delete remaining old lines
        (Some(&line), None) => {
            let mut rest = myers_rec(old, new, old_idx + 1, new_idx, memo)
                .as_ref()
                .clone();
            rest.insert(0, Edit::Delete(line));
            rest
        }
        // Lines match, keep and advance both
        (Some(&old_line), Some(&new_line)) if old_line == new_line => {
            let mut rest = myers_rec(old, new, old_idx + 1, new_idx + 1, memo)
                .as_ref()
                .clone();
            rest.insert(0, Edit::Keep(old_line));
            rest
        }
        // Lines differ, pick whichever of delete/insert leads to fewer edits
        (Some(&old_line), Some(&new_line)) => {
            let delete = myers_rec(old, new, old_idx + 1, new_idx, memo);
            let insert = myers_rec(old, new, old_idx, new_idx + 1, memo);
            if delete.len() <= insert.len() {
                let mut rest = delete.as_ref().clone();
                rest.insert(0, Edit::Delete(old_line));
                rest
            } else {
                let mut rest = insert.as_ref().clone();
                rest.insert(0, Edit::Insert(new_line));
                rest
            }
        }
    };

    let result = Rc::new(result);
    memo.insert((old_idx, new_idx), Rc::clone(&result));
    result
}
