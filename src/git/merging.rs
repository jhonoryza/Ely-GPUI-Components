use similar::{MergeResolution, TextMerge};

/// Which side a conflict takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Take {
    Ours,
    Theirs,
    Both,
}

/// How a merge region stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionKind {
    Unchanged,
    Ours,
    Theirs,
    /// Both sides made the same change.
    Both,
    Conflict,
}

/// A stretch of a three-way merge, with each side's lines.
#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    pub kind: RegionKind,
    pub base: Vec<String>,
    pub ours: Vec<String>,
    pub theirs: Vec<String>,
}

fn lines(text: &str, range: std::ops::Range<usize>) -> Vec<String> {
    text.lines()
        .skip(range.start)
        .take(range.len())
        .map(str::to_string)
        .collect()
}

/// `ours` and `theirs` merged against `base`, region by region.
pub fn regions(base: &str, ours: &str, theirs: &str) -> Vec<Region> {
    TextMerge::from_lines(base, ours, theirs)
        .regions()
        .iter()
        .map(|region| Region {
            kind: match region.resolution() {
                MergeResolution::Unchanged => RegionKind::Unchanged,
                MergeResolution::Ours => RegionKind::Ours,
                MergeResolution::Theirs => RegionKind::Theirs,
                MergeResolution::Both => RegionKind::Both,
                MergeResolution::Conflict => RegionKind::Conflict,
                other => unreachable!("a merge resolution this crate does not know: {other:?}"),
            },
            base: lines(base, region.base_range()),
            ours: lines(ours, region.ours_range()),
            theirs: lines(theirs, region.theirs_range()),
        })
        .collect()
}

/// The merged text with each conflict as taken, and how many are still open; an open one keeps its markers.
pub fn result(regions: &[Region], takes: &[Option<Take>]) -> (String, usize) {
    let mut out: Vec<String> = Vec::new();
    let mut conflicts = 0;
    let mut open = 0;
    for region in regions {
        match region.kind {
            RegionKind::Unchanged | RegionKind::Ours | RegionKind::Both => {
                out.extend(region.ours.iter().cloned())
            }
            RegionKind::Theirs => out.extend(region.theirs.iter().cloned()),
            RegionKind::Conflict => {
                match takes.get(conflicts).copied().flatten() {
                    Some(Take::Ours) => out.extend(region.ours.iter().cloned()),
                    Some(Take::Theirs) => out.extend(region.theirs.iter().cloned()),
                    Some(Take::Both) => {
                        out.extend(region.ours.iter().cloned());
                        out.extend(region.theirs.iter().cloned());
                    }
                    None => {
                        open += 1;
                        out.push("<<<<<<< ours".into());
                        out.extend(region.ours.iter().cloned());
                        out.push("=======".into());
                        out.extend(region.theirs.iter().cloned());
                        out.push(">>>>>>> theirs".into());
                    }
                }
                conflicts += 1;
            }
        }
    }
    let mut text = out.join("\n");
    if !out.is_empty() {
        text.push('\n');
    }
    (text, open)
}

/// A conflict in marked text: the lines of its markers, from zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub start: usize,
    pub middle: usize,
    pub end: usize,
}

/// The conflicts marked in `text`: `<<<<<<<`, `=======`, `>>>>>>>`, each at a line's start.
pub fn conflicts(text: &str) -> Vec<Conflict> {
    let mut found = Vec::new();
    let (mut start, mut middle) = (None, None);
    for (ix, line) in text.lines().enumerate() {
        if line.starts_with("<<<<<<<") {
            (start, middle) = (Some(ix), None);
        } else if line.starts_with("=======") && start.is_some() {
            middle = Some(ix);
        } else if line.starts_with(">>>>>>>")
            && let (Some(start), Some(middle)) = (start.take(), middle.take())
        {
            found.push(Conflict {
                start,
                middle,
                end: ix,
            });
        }
    }
    found
}

/// `text` with conflict `ix` settled as taken.
pub fn resolve(text: &str, ix: usize, take: Take) -> String {
    let conflict = conflicts(text)[ix];
    let lines: Vec<&str> = text.lines().collect();
    let ours = &lines[conflict.start + 1..conflict.middle];
    let theirs = &lines[conflict.middle + 1..conflict.end];
    let kept: Vec<&str> = match take {
        Take::Ours => ours.to_vec(),
        Take::Theirs => theirs.to_vec(),
        Take::Both => ours.iter().chain(theirs).copied().collect(),
    };
    let mut out: Vec<&str> = lines[..conflict.start].to_vec();
    out.extend(kept);
    out.extend(&lines[conflict.end + 1..]);
    let mut settled = out.join("\n");
    if text.ends_with('\n') {
        settled.push('\n');
    }
    log::info!("conflict {ix} took {take:?}");
    settled
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "fn lift() {\n    0.5\n}\nfn keep() {}\n";
    const OURS: &str = "fn lift() {\n    0.6\n}\nfn keep() {}\n";
    const THEIRS: &str = "fn lift() {\n    0.75\n}\nfn keep() { log() }\n";

    #[test]
    fn a_merge_keeps_clean_changes_and_holds_conflicts() {
        let all = regions(BASE, OURS, THEIRS);
        let kinds: Vec<RegionKind> = all.iter().map(|region| region.kind).collect();
        assert!(kinds.contains(&RegionKind::Conflict) && kinds.contains(&RegionKind::Theirs));
        let (open, count) = result(&all, &[]);
        assert_eq!(count, 1);
        assert!(open.contains("<<<<<<< ours\n    0.6\n=======\n    0.75\n>>>>>>> theirs"));
        let (taken, count) = result(&all, &[Some(Take::Theirs)]);
        assert_eq!(count, 0);
        assert_eq!(taken, "fn lift() {\n    0.75\n}\nfn keep() { log() }\n");
    }

    #[test]
    fn markers_resolve_to_one_side_or_both() {
        let marked = "a\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> topic\nz\n";
        assert_eq!(
            conflicts(marked),
            [Conflict {
                start: 1,
                middle: 3,
                end: 5
            }]
        );
        assert_eq!(resolve(marked, 0, Take::Ours), "a\nours\nz\n");
        assert_eq!(resolve(marked, 0, Take::Both), "a\nours\ntheirs\nz\n");
        assert!(
            conflicts("a\n=======\nb\n").is_empty(),
            "a rule alone is no conflict"
        );
    }
}
