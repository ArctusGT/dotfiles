//! What fzf shows of the frames `claude-sidebar` draws: after each gather
//! the same frame of the new rows, and at every tick the next frame while
//! one differs from another, so a Clawd moves. fzf reloads only when the
//! rows on show change: with none moving, when the rows do; with one, at a
//! tick, unless the frame repeats the one before it and fzf already has
//! this gather's rows. Its cursor moves only when the right pane's session
//! did, so it is not taken from an entry being browsed to.

/// The sidebar's cache, as `Serve` reads it, and its fzf.
pub trait Sidebar {
    /// the rows of frame `frame`: NUL-terminated fzf items of tab fields
    fn rows(&self, frame: usize) -> Vec<u8>;
    /// for each frame, whether it is the same as the frame before it
    /// (frame 0 as the last)
    fn repeats(&self) -> Vec<bool>;
    /// the value `jumped` holds, which each jump in the frame replaces
    fn jumped(&self) -> String;
    /// whether a jump is moving the right pane's client now
    fn moving(&self) -> bool;
    /// has fzf show the rows of frame `frame`; succeeds if fzf took it
    fn reload(&mut self, frame: usize) -> bool;
    /// moves fzf's cursor to item `position`, from 1; succeeds if fzf took it
    fn move_cursor(&mut self, position: usize) -> bool;
}

/// A gather that has drawn new rows.
pub struct Gather {
    /// the value `jumped` had when the gather began
    pub since: String,
    /// whether a kick came during it, so what it saw on show may already be
    /// gone
    pub moved: bool,
}

/// the tab field (from 0) holding 1 on the item the frame's right pane shows
const SHOWN: usize = 5;

pub struct Serve<S> {
    sidebar: S,
    frame: usize,
    repeats: Vec<bool>,
    /// whether fzf has rows of the latest gather
    fzf_has_rows: bool,
    /// whether the cursor still has to follow the latest gather
    check_shown: bool,
    /// the value `jumped` had when the latest gather began
    since: String,
    /// the rows fzf shows, less the session on show, while none moves
    last_rows: Option<Vec<u8>>,
    last_shown: Option<usize>,
}

impl<S: Sidebar> Serve<S> {
    pub fn new(sidebar: S) -> Serve<S> {
        Serve {
            sidebar,
            frame: 0,
            repeats: Vec::new(),
            fzf_has_rows: false,
            check_shown: false,
            since: String::new(),
            last_rows: None,
            last_shown: None,
        }
    }

    /// whether a Clawd moves: a frame not repeating the one before it
    pub fn animating(&self) -> bool {
        self.repeats.contains(&false)
    }

    /// shows what `gather` drew
    pub fn gathered(&mut self, gather: Gather) {
        self.repeats = self.sidebar.repeats();
        self.since = gather.since;
        self.check_shown = !gather.moved;
        // with one moving, the next tick shows the new rows, so the frames
        // stay evenly spaced
        if self.animating() {
            self.fzf_has_rows = false;
        } else {
            let rows = without_shown(&self.sidebar.rows(self.frame));
            if self.last_rows.as_ref() != Some(&rows) && self.sidebar.reload(self.frame) {
                self.last_rows = Some(rows);
            }
            self.fzf_has_rows = true;
        }
        self.follow();
    }

    /// shows the next frame, if it changes what fzf shows
    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % self.repeats.len();
        if !self.fzf_has_rows || !self.repeats[self.frame] {
            self.sidebar.reload(self.frame);
        }
        self.last_rows = None;
        self.fzf_has_rows = true;
        self.follow();
    }

    /// moves the cursor to the session on show, once fzf has the rows its
    /// position is counted in, and not for a gather a jump came after, or
    /// while one moves: a gather that began before a jump saw the session on
    /// show before it, and would take the cursor back from where the jump
    /// put it. Checked just before the post, so a pick cannot land between
    fn follow(&mut self) {
        if !(self.check_shown && self.fzf_has_rows) {
            return;
        }
        self.check_shown = false;
        let Some(shown) = position(&self.sidebar.rows(0), SHOWN, b"1") else {
            return;
        };
        if Some(shown) != self.last_shown
            && self.sidebar.jumped() == self.since
            && !self.sidebar.moving()
            && self.sidebar.move_cursor(shown)
        {
            self.last_shown = Some(shown);
        }
    }
}

/// `rows` with the session-on-show field emptied in every item, as it moves
/// the cursor, not the rows
fn without_shown(rows: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rows.len());
    for item in rows.split_inclusive(|&b| b == 0) {
        let mut tabs = item
            .iter()
            .enumerate()
            .filter(|&(_, &b)| b == b'\t')
            .map(|(i, _)| i);
        match (tabs.nth(SHOWN - 1), tabs.next()) {
            (Some(start), Some(end)) => {
                out.extend_from_slice(&item[..=start]);
                out.extend_from_slice(&item[end..]);
            }
            _ => out.extend_from_slice(item),
        }
    }
    out
}

/// the position in fzf's list, from 1, of the first item whose tab field
/// `field` (from 0) is `value`
fn position(rows: &[u8], field: usize, value: &[u8]) -> Option<usize> {
    rows.split(|&b| b == 0)
        .position(|item| item.split(|&b| b == b'\t').nth(field) == Some(value))
        .map(|i| i + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// a sidebar of `items` per frame, recording what fzf is sent
    #[derive(Default)]
    struct Fake {
        frames: Vec<Vec<u8>>,
        repeats: Vec<bool>,
        jumped: String,
        moving: bool,
        sent: Vec<String>,
    }

    impl Sidebar for Fake {
        fn rows(&self, frame: usize) -> Vec<u8> {
            self.frames[frame].clone()
        }
        fn repeats(&self) -> Vec<bool> {
            self.repeats.clone()
        }
        fn jumped(&self) -> String {
            self.jumped.clone()
        }
        fn moving(&self) -> bool {
            self.moving
        }
        fn reload(&mut self, frame: usize) -> bool {
            self.sent.push(format!("reload {frame}"));
            true
        }
        fn move_cursor(&mut self, position: usize) -> bool {
            self.sent.push(format!("pos {position}"));
            true
        }
    }

    /// an item: host, pane, relay, id, number, shown, then its text
    fn item(n: u32, shown: bool, text: &str) -> String {
        format!(
            "\t%{n}\t\tid{n}\t{n}\t{}\t{text}\0",
            if shown { "1" } else { "" }
        )
    }

    /// `count` frames of three items with the second on show; `moves`
    /// gives frame k its own text, else every frame is the same
    fn sidebar(count: usize, moves: bool) -> Fake {
        let frames = (0..count)
            .map(|k| {
                let text = if moves {
                    format!("frame {k}")
                } else {
                    "still".into()
                };
                [
                    item(1, false, "a"),
                    item(2, true, &text),
                    item(3, false, "c"),
                ]
                .concat()
                .into_bytes()
            })
            .collect();
        Fake {
            frames,
            repeats: vec![!moves; count],
            jumped: "j1".into(),
            ..Fake::default()
        }
    }

    fn gather(since: &str) -> Gather {
        Gather {
            since: since.into(),
            moved: false,
        }
    }

    fn sent(serve: &mut Serve<Fake>) -> Vec<String> {
        std::mem::take(&mut serve.sidebar.sent)
    }

    #[test]
    fn still_rows_reload_only_when_they_change() {
        let mut serve = Serve::new(sidebar(4, false));
        serve.gathered(gather("j1"));
        assert_eq!(sent(&mut serve), ["reload 0", "pos 2"]);
        assert!(!serve.animating());
        serve.gathered(gather("j1"));
        assert!(sent(&mut serve).is_empty());
        serve.sidebar.frames[0] = [item(1, false, "a"), item(2, true, "new")].concat().into();
        serve.gathered(gather("j1"));
        assert_eq!(sent(&mut serve), ["reload 0"]);
    }

    #[test]
    fn the_session_on_show_moves_the_cursor_not_the_rows() {
        let mut serve = Serve::new(sidebar(1, false));
        serve.gathered(gather("j1"));
        sent(&mut serve);
        serve.sidebar.frames[0] = [
            item(1, true, "a"),
            item(2, false, "still"),
            item(3, false, "c"),
        ]
        .concat()
        .into();
        serve.gathered(gather("j1"));
        assert_eq!(sent(&mut serve), ["pos 1"]);
    }

    #[test]
    fn a_moving_clawd_reloads_at_each_tick_not_at_the_gather() {
        let mut serve = Serve::new(sidebar(4, true));
        serve.gathered(gather("j1"));
        assert!(serve.animating());
        assert!(sent(&mut serve).is_empty());
        for _ in 0..5 {
            serve.tick();
        }
        assert_eq!(
            sent(&mut serve),
            [
                "reload 1", "pos 2", "reload 2", "reload 3", "reload 0", "reload 1"
            ]
        );
    }

    #[test]
    fn a_repeated_frame_reloads_only_for_a_new_gather() {
        let mut serve = Serve::new(sidebar(4, true));
        serve.sidebar.repeats = vec![false, true, false, true];
        serve.gathered(gather("j1"));
        serve.tick();
        serve.tick();
        serve.tick();
        assert_eq!(sent(&mut serve), ["reload 1", "pos 2", "reload 2"]);
        serve.gathered(gather("j1"));
        serve.tick();
        assert_eq!(sent(&mut serve), ["reload 0"]);
    }

    #[test]
    fn the_cursor_stays_for_a_gather_a_jump_overtook() {
        let mut serve = Serve::new(sidebar(1, false));
        serve.sidebar.jumped = "j2".into();
        serve.gathered(gather("j1"));
        assert_eq!(sent(&mut serve), ["reload 0"]);
        serve.sidebar.jumped = "j1".into();
        serve.gathered(Gather {
            since: "j1".into(),
            moved: true,
        });
        assert!(sent(&mut serve).is_empty());
        serve.sidebar.moving = true;
        serve.gathered(gather("j1"));
        assert!(sent(&mut serve).is_empty());
        serve.sidebar.moving = false;
        serve.gathered(gather("j1"));
        assert_eq!(sent(&mut serve), ["pos 2"]);
    }

    #[test]
    fn the_cursor_is_left_alone_while_the_session_on_show_stays() {
        let mut serve = Serve::new(sidebar(1, false));
        serve.gathered(gather("j1"));
        sent(&mut serve);
        serve.gathered(gather("j1"));
        assert!(sent(&mut serve).is_empty());
    }

    #[test]
    fn position_counts_items_from_one() {
        let rows = [item(1, false, "a"), item(2, false, "b"), item(3, true, "c")].concat();
        assert_eq!(position(rows.as_bytes(), SHOWN, b"1"), Some(3));
        assert_eq!(position(rows.as_bytes(), 4, b"2"), Some(2));
        assert_eq!(position(rows.as_bytes(), 4, b"9"), None);
    }

    #[test]
    fn without_shown_empties_only_that_field() {
        let rows = format!("{}header\0", item(1, true, "a\tb"));
        assert_eq!(
            without_shown(rows.as_bytes()),
            b"\t%1\t\tid1\t1\t\ta\tb\0header\0"
        );
    }
}
