//! The sidebar's frames, rendered from a gather's hosts (`gather`): one
//! NUL-terminated fzf item per host heading or session, the same items in
//! every frame but for Clawd's pose.
//!
//! A session's item is its tab fields: ssh host, pane, relay session,
//! session id (empty only on a header, which is how `jump` tells one), entry
//! number, 1 if the frame's right pane shows it, its repo if it leads one
//! (what `claude-sidebar pin` pins), then its display lines.
//! A host's sessions come grouped by repo (`gather`'s `listed`). The first
//! of each group leads it, and is numbered if it has a pane: by its repo's
//! pin, else the next number, in order across hosts, that no pin holds;
//! `pick N` finds entry N by its number field in `rows.0`. The rest of the
//! group are indented under it, unnumbered.
//! Times are in whole minutes, so the rows change only when a session does,
//! or with the frame while a Clawd moves.

use crate::gather::Host;
use crate::pins::Pins;
use crate::session::{Number, Session, State};

/// the frames a gather renders: every animation (running, squatting,
/// waiting) loops within them, as each one's length divides it
pub const FRAME_COUNT: usize = 48;

/// what the frames are rendered for, besides the sessions
pub struct View {
    /// the columns an item's line has, past fzf's pointer and marker
    pub width: i64,
    /// the context percentage a context line turns red at
    pub warn: f64,
    /// the time, in seconds since the epoch, the times are counted to
    pub now: i64,
    /// the entry numbers pinned to repos
    pub pins: Pins,
}

// TokyoNight Night as SGR foregrounds: the terminal palette where it has the
// shade (palette N is 30+N, or 90+N-8 for 8..15), hex where it does not
const RED: &str = "31";
const GREEN: &str = "32";
const YELLOW: &str = "33";
const BLUE: &str = "34";
/// palette 8, #414868
const TERMINAL_BLACK: &str = "90";
/// #ff9e64
const ORANGE: &str = "38;2;255;158;100";
/// #737aa2, a grey still seen dimmed, where palette 8 is lost
const DARK5: &str = "38;2;115;122;162";
/// shades a bandana takes, clear of the state colours: red, green, blue,
/// magenta, cyan, teal, fg, blue1, purple
const BANDANA_COLOURS: [&str; 9] = [
    RED,
    GREEN,
    BLUE,
    "35",
    "36",
    "38;2;115;218;202",
    "38;2;192;202;245",
    "38;2;42;195;222",
    "38;2;157;124;216",
];
/// the background, #1a1b26, that `faded` takes a colour halfway to
const BACKGROUND: [u32; 3] = [26, 27, 38];

/// the cells an entry under its group's first is moved right by: enough to
/// see, little enough to keep the width
const INDENT: &str = "  ";

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// every frame of `hosts`' items, frame 0 to `frames - 1`, each frame's
/// items one after another
pub fn render(hosts: &[Host], view: &View, frames: usize) -> Vec<String> {
    let clawd = Clawd::new();
    let mut out = vec![String::new(); frames];
    // the last number given to a repo with no pin
    let mut unpinned = 0;
    for host in hosts {
        let item = header(host);
        for frame in &mut out {
            frame.push_str(&item);
        }
        let mut group = None;
        for session in &host.sessions {
            let leads = group != Some(&session.repo);
            group = Some(&session.repo);
            let pin = session
                .repo
                .as_deref()
                .and_then(|repo| view.pins.number(&host.ssh_host, repo));
            let number = (leads && session.pane.is_some()).then(|| {
                pin.unwrap_or_else(|| {
                    unpinned += 1;
                    while view.pins.holds(unpinned) {
                        unpinned += 1;
                    }
                    unpinned
                })
            });
            render_session(session, host, number, !leads, view, &clawd, &mut out);
        }
    }
    out
}

/// appends `session`'s item, entry `number` if it has one, to each of
/// `frames`
fn render_session(
    session: &Session,
    host: &Host,
    number: Option<usize>,
    indented: bool,
    view: &View,
    clawd: &Clawd,
    frames: &mut [String],
) {
    let entry = Entry::new(session, host, number, indented, view);
    let (animation, phase) = clawd.animation(session);
    let palette = Palette::new(session);
    let faint = session.pane.is_none() || indented;
    // each distinct pose drawn once, as most repeat across the frames
    // or, standing, are the same in all of them
    let mut drawn: Vec<Option<[String; 3]>> = vec![None; clawd.poses.len()];
    for (k, frame) in frames.iter_mut().enumerate() {
        let pose_id = animation[(phase + k) % animation.len()];
        let picture =
            drawn[pose_id].get_or_insert_with(|| picture(&clawd.poses[pose_id], &palette, faint));
        entry.write(frame, picture);
    }
}

/// a host's heading: its label, `unreachable` if it did not answer, then a
/// blank second line, so the heading stands apart from the entries
fn header(host: &Host) -> String {
    // paint closes only the colour, so the reset ends the bold
    let mut item = format!(
        "\t\t\t\t\t\t\t{}{RESET}",
        paint(&host.label, &format!("1;{BLUE}"))
    );
    if host.down {
        item += &format!("  {}", paint("unreachable", RED));
    }
    // a braille blank (U+2800), since fzf drops a trailing line of spaces
    item + "\n\u{2800}\0"
}

/// a session's item, less the picture: what goes before and right of each
/// of its three lines
struct Entry {
    /// the tab fields, through the tab before the display lines
    fields: String,
    /// before each line: the cells an entry under its group's first is
    /// moved right by
    indent: &'static str,
    /// dims each line of a session with no pane (a --bg one, or a daemon),
    /// and of one indented under its repo's first, which it would take the
    /// place of when selected as a background one does
    dim: &'static str,
    /// right of the picture on each line: number and repo, name, then
    /// status, the last line also carrying the context line below it
    texts: [String; 3],
}

impl Entry {
    fn new(
        session: &Session,
        host: &Host,
        number: Option<usize>,
        indented: bool,
        view: &View,
    ) -> Entry {
        let has_pane = session.pane.is_some();
        let indent = if indented { INDENT } else { "" };
        let on_show = host.on_show.is_some() && host.on_show == session.pane;
        let number_text = number.map(|n| n.to_string());
        let fields = [
            host.ssh_host.clone(),
            session.pane.clone().unwrap_or_default(),
            host.relay.clone(),
            session.id.clone(),
            number_text.clone().unwrap_or_default(),
            if on_show { "1" } else { "" }.to_string(),
            if indented {
                String::new()
            } else {
                session.repo.clone().unwrap_or_default()
            },
            String::new(),
        ]
        .join("\t");
        let dim = if has_pane && !indented { "" } else { DIM };
        // the repo the entry is grouped by, its cwd from a host whose
        // claude-sessions gives no repo
        let repo = session.repo.as_deref().unwrap_or(&session.cwd);
        let repo_text = format!(
            " {} {}\n",
            number_text.as_deref().unwrap_or(" "),
            last_part(repo)
        );
        let name_text = format!(
            "   {}\n",
            session.name.as_deref().unwrap_or_else(|| short_id(&session.id))
        );
        let state = session.state.word();
        let age = ago(view.now as f64 - session.since);
        // running subagents in the working colour, so an idle session whose
        // subagents are still at work stands out
        let subagents = &session.subagents;
        let subagents_label = if session.subagents_running() {
            format!("  {}", paint(&format!("+{}", subagents.text), ORANGE))
        } else {
            String::new()
        };
        let background_label = if has_pane {
            String::new()
        } else {
            format!("  {DIM}background{RESET}")
        };
        let width = view.width - indent.chars().count() as i64;
        let context = context_line(session.used_pct.as_ref(), width, view);
        // reset at the end, or the dim of a background entry leaks into the
        // entries fzf draws after it
        let status_text = format!(
            "   {state} {age}{subagents_label}{background_label}\n{indent}{dim}{context}{RESET}"
        );
        Entry {
            fields,
            indent,
            dim,
            texts: [repo_text, name_text, status_text],
        }
    }

    /// appends the item, with `picture` left of its lines, to `frame`
    fn write(&self, frame: &mut String, picture: &[String; 3]) {
        frame.push_str(&self.fields);
        for (line, text) in picture.iter().zip(&self.texts) {
            frame.push_str(self.indent);
            frame.push_str(self.dim);
            frame.push_str(line);
            frame.push_str(text);
        }
        frame.push('\0');
    }
}

/// the last part of path `cwd`
fn last_part(cwd: &str) -> &str {
    cwd.rsplit('/').next().unwrap_or_default()
}

/// the first 8 characters of `id`
fn short_id(id: &str) -> &str {
    id.char_indices().nth(8).map_or(id, |(end, _)| &id[..end])
}

/// `seconds` as whole minutes or hours
fn ago(seconds: f64) -> String {
    if seconds < 60.0 {
        "<1m".into()
    } else if seconds < 3600.0 {
        format!("{}m", (seconds / 60.0).floor() as i64)
    } else {
        format!("{}h", (seconds / 3600.0).floor() as i64)
    }
}

/// `text` in the colour `sgr`, then back to the default foreground (39)
/// rather than a reset, so a background entry stays dimmed
fn paint(text: &str, sgr: &str) -> String {
    format!("\x1b[{sgr}m{text}\x1b[39m")
}

/// `text` in the foreground `fg` on the background `bg`, both given as
/// foregrounds, then back to the default colours
fn paint_on(text: &str, fg: &str, bg: &str) -> String {
    format!("\x1b[{fg};{}m{text}\x1b[39;49m", as_background(bg))
}

/// foreground `sgr` as the same colour for the background: 3x as 4x, 9x as
/// 10x, 38;2 as 48;2
fn as_background(sgr: &str) -> String {
    match sgr.strip_prefix("38;") {
        Some(rest) => format!("48;{rest}"),
        // the palette shades, all numbers
        None => sgr
            .parse::<u32>()
            .map_or(String::new(), |n| (n + 10).to_string()),
    }
}

/// foreground `sgr` halfway to the background, as Ghostty draws a faint
/// (SGR 2) foreground at its faint-opacity of 0.5; the palette shades as
/// TokyoNight Night sets them
fn faded(sgr: &str) -> String {
    let rgb: Vec<u32> = match sgr.strip_prefix("38;2;") {
        Some(rgb) => rgb.split(';').filter_map(|c| c.parse().ok()).collect(),
        None => match sgr {
            "31" => vec![247, 118, 142],
            "32" => vec![158, 206, 106],
            "33" => vec![224, 175, 104],
            "34" => vec![122, 162, 247],
            "35" => vec![187, 154, 247],
            "36" => vec![125, 207, 255],
            // never taken: only the bandana and body colours are faded
            _ => return sgr.into(),
        },
    };
    let halves: Vec<String> = rgb
        .iter()
        .zip(BACKGROUND)
        .map(|(c, bg)| ((c + bg) / 2).to_string())
        .collect();
    format!("38;2;{}", halves.join(";"))
}

/// context use as a line across the entry, `width` columns with the
/// percentage label at its end: the used share a heavy rule, each cell
/// coloured by how far along the bar it sits, green to red as a full context
/// is the bad end. The rest a light dashed rule in terminal_black. All red at
/// `view.warn`. No record (`pct` none): the whole line terminal_black, with
/// no label
fn context_line(pct: Option<&Number>, width: i64, view: &View) -> String {
    let Some(Number { text, value: share }) = pct else {
        return paint(&"┈".repeat(width.max(0) as usize), TERMINAL_BLACK);
    };
    let share = *share;
    // padded to 4 columns, so bars up to 99% end in the same column
    let label = format!("{:>4}", format!(" {text}%"));
    let cells = width - label.chars().count() as i64;
    let full = (share * cells as f64 / 100.0).round().min(cells as f64) as i64;
    let warned = share >= view.warn;
    let palette: &[&str] = if warned {
        &[RED]
    } else {
        &[GREEN, YELLOW, ORANGE, RED]
    };
    let colours: Vec<&str> = (0..full)
        .map(|cell| palette[(cell as f64 * palette.len() as f64 / cells as f64).floor() as usize])
        .collect();
    // painted a run of one colour at a time
    let used: String = colours
        .chunk_by(|a, b| a == b)
        .map(|run| paint(&"━".repeat(run.len()), run[0]))
        .collect();
    let rest = paint(&"┈".repeat((cells - full).max(0) as usize), TERMINAL_BLACK);
    let label = if warned { paint(&label, RED) } else { label };
    used + &rest + &label
}

/// djb2 of a string, kept under 2^31 as jq, counting in doubles, kept it
fn djb2(s: &str) -> u64 {
    s.chars()
        .fold(5381, |h, c| (h * 33 + c as u64) % 2147483647)
}

/// the colours Clawd is drawn in for a session, by the letters of a pose:
/// B his body, in the colour for its state (working orange, waiting yellow,
/// idle grey), m its bandana, 1 2 3 the bandana's stripes
struct Palette {
    body: &'static str,
    /// main then stripes: four different colours from a stable hash of the
    /// session id, so a session wears the same bandana on every refresh and
    /// from either host. Each pick takes the hash modulo the colours left
    /// and passes the quotient on to the next
    bandana: [&'static str; 4],
}

impl Palette {
    fn new(session: &Session) -> Palette {
        let body = match session.state {
            State::Working => ORANGE,
            State::Waiting => YELLOW,
            State::Idle => DARK5,
        };
        let mut hash = djb2(&session.id);
        let mut left = BANDANA_COLOURS.to_vec();
        let bandana = std::array::from_fn(|_| {
            let n = left.len() as u64;
            let pick = left.remove((hash % n) as usize);
            hash /= n;
            pick
        });
        Palette { body, bandana }
    }

    fn colour(&self, letter: u8) -> Option<&'static str> {
        match letter {
            b'B' => Some(self.body),
            b'm' => Some(self.bandana[0]),
            b'1' => Some(self.bandana[1]),
            b'2' => Some(self.bandana[2]),
            b'3' => Some(self.bandana[3]),
            _ => None,
        }
    }
}

/// Clawd in `pose` as three lines of octant cells, 2 pixels wide and 4 tall
fn picture(pose: &Pose, palette: &Palette, faint: bool) -> [String; 3] {
    std::array::from_fn(|line| {
        (0..8)
            .map(|cell| {
                octant(
                    cell_pixels(pose, line, cell).map(|p| palette.colour(p)),
                    faint,
                )
            })
            .collect()
    })
}

/// the 8 pixels of octant cell `cell` on picture line `line`, left to right,
/// top to bottom
fn cell_pixels(pose: &Pose, line: usize, cell: usize) -> [u8; 8] {
    std::array::from_fn(|k| pose[4 * line + k / 2][2 * cell + k % 2])
}

/// the octant for each set of lit pixels in a cell, bit n-1 for pixel n:
/// 1 2 / 3 4 / 5 6 / 7 8. Written out from Unicode names by Python 3.15
/// unicodedata: BLOCK OCTANT-n at U+1CD00 lights the pixels its digits name,
/// and the 26 sets Unicode encodes elsewhere are the quadrants, U+1FB82,
/// U+2582, U+1FB85, U+2586, U+1CEA8, U+1CEAB, U+1CEA3, U+1CEA0, U+1FBE6 and
/// U+1FBE7
const OCTANTS: &str = concat!(
    " 𜺨𜺫🮂𜴀▘𜴁𜴂𜴃𜴄▝𜴅𜴆𜴇𜴈▀",
    "𜴉𜴊𜴋𜴌🯦𜴍𜴎𜴏𜴐𜴑𜴒𜴓𜴔𜴕𜴖𜴗",
    "𜴘𜴙𜴚𜴛𜴜𜴝𜴞𜴟🯧𜴠𜴡𜴢𜴣𜴤𜴥𜴦",
    "𜴧𜴨𜴩𜴪𜴫𜴬𜴭𜴮𜴯𜴰𜴱𜴲𜴳𜴴𜴵🮅",
    "𜺣𜴶𜴷𜴸𜴹𜴺𜴻𜴼𜴽𜴾𜴿𜵀𜵁𜵂𜵃𜵄",
    "▖𜵅𜵆𜵇𜵈▌𜵉𜵊𜵋𜵌▞𜵍𜵎𜵏𜵐▛",
    "𜵑𜵒𜵓𜵔𜵕𜵖𜵗𜵘𜵙𜵚𜵛𜵜𜵝𜵞𜵟𜵠",
    "𜵡𜵢𜵣𜵤𜵥𜵦𜵧𜵨𜵩𜵪𜵫𜵬𜵭𜵮𜵯𜵰",
    "𜺠𜵱𜵲𜵳𜵴𜵵𜵶𜵷𜵸𜵹𜵺𜵻𜵼𜵽𜵾𜵿",
    "𜶀𜶁𜶂𜶃𜶄𜶅𜶆𜶇𜶈𜶉𜶊𜶋𜶌𜶍𜶎𜶏",
    "▗𜶐𜶑𜶒𜶓▚𜶔𜶕𜶖𜶗▐𜶘𜶙𜶚𜶛▜",
    "𜶜𜶝𜶞𜶟𜶠𜶡𜶢𜶣𜶤𜶥𜶦𜶧𜶨𜶩𜶪𜶫",
    "▂𜶬𜶭𜶮𜶯𜶰𜶱𜶲𜶳𜶴𜶵𜶶𜶷𜶸𜶹𜶺",
    "𜶻𜶼𜶽𜶾𜶿𜷀𜷁𜷂𜷃𜷄𜷅𜷆𜷇𜷈𜷉𜷊",
    "𜷋𜷌𜷍𜷎𜷏𜷐𜷑𜷒𜷓𜷔𜷕𜷖𜷗𜷘𜷙𜷚",
    "▄𜷛𜷜𜷝𜷞▙𜷟𜷠𜷡𜷢▟𜷣▆𜷤𜷥█",
);

/// a cell of 8 pixels (colours or none, left to right, top to bottom) as its
/// octant: the first lit pixel is the ink and lights the glyph, and a second
/// colour filling the rest of the cell is its background, closed again (49)
/// so it stops at the cell. A second colour in a cell that is not full is
/// not drawn. Where `faint`, a cell turns faint off (22), draws its colours
/// faded by hand, and turns faint back on, rather than leaving the dimming
/// to SGR 2: faint dims only a foreground, and fzf draws its current line
/// bold, under which Ghostty keeps faint on some glyphs and drops it from
/// others, so a sprite dimmed by SGR 2 comes out patchy when selected
fn octant(pixels: [Option<&str>; 8], faint: bool) -> String {
    let lit: Vec<&str> = pixels.iter().flatten().copied().collect();
    let Some(&ink) = lit.first() else {
        return " ".into();
    };
    let background = if lit.len() == 8 {
        lit.iter().copied().find(|&c| c != ink)
    } else {
        None
    };
    let bits = (0..8)
        .filter(|&k| pixels[k] == Some(ink))
        .fold(0, |bits, k| bits | 1 << k);
    let glyph = OCTANTS.chars().nth(bits).unwrap_or(' ').to_string();
    match (background, faint) {
        (None, false) => paint(&glyph, ink),
        (None, true) => format!("\x1b[22m{}{DIM}", paint(&glyph, &faded(ink))),
        (Some(bg), false) => paint_on(&glyph, ink, bg),
        (Some(bg), true) => {
            format!("\x1b[22m{}{DIM}", paint_on(&glyph, &faded(ink), &faded(bg)))
        }
    }
}

/// a picture of Clawd 16 pixels wide and 12 tall, a row of letters each: B
/// body, m bandana, 1 2 3 its stripes, . empty. Drawn as octant cells 2
/// pixels wide and 4 tall, from rows 0, 4 and 8, so each cell keeps to two
/// colours, the empty background counting as one: `octant` draws a cell as
/// one colour and empty, or as two colours and full
type Pose = [[u8; 16]; 12];

fn pose(rows: [&str; 12]) -> Pose {
    rows.map(|row| row.as_bytes().try_into().expect("a pose row is 16 pixels"))
}

/// Clawd standing, as Claude Code 2.1.292 draws him in its banner, open eyes
/// and arms down, with a bandana below his head-top: the ribbon a tail at its
/// end and a drop beside the eyes. Two rows each for head-top, bandana, arms,
/// body and feet; the eyes slits one row tall, under the bandana
const STAND: [&str; 12] = [
    "..BBBBBBBBBBBB..",
    "..BBBBBBBBBBBB..",
    "..112233mmmmmm3.",
    "..112233mmmmmm3.",
    "..BB..BBBB..BB22",
    "..BBBBBBBBBBBB22",
    "BBBBBBBBBBBBBBBB",
    "BBBBBBBBBBBBBBBB",
    "..BBBBBBBBBBBB..",
    "..BBBBBBBBBBBB..",
    "..B.B......B.B..",
    "..B.B......B.B..",
];

/// Clawd running: body two columns right, slit eyes, both arms swept left,
/// the stripes at the far end of the bandana. Columns 0 to 3 of rows 0 to 3,
/// two octant cells, hold the ribbon streaming left, `RIBBONS[k]` in frame k
/// of 4, each cell in one stripe colour that changes each frame. The eyes
/// glance a pixel with each step, `RUN_EYES[k]`: centre, a pixel right,
/// centre, a pixel right. The feet step left pair down, all up, right pair
/// down, all up
const RUN_BODY: [&str; 10] = [
    "....BBBBBBBBBBBB",
    "....BBBBBBBBBBBB",
    "....mmmmmm332211",
    "....mmmmmm332211",
    "....BB..BBBB..BB",
    ".BB.BBBBBBBBBBBB",
    ".BB.BBBBBBBBBBBB",
    "....BBBBBBBBBBBB",
    ".BBBBBBBBBBBBBBB",
    ".BBBBBBBBBBBBBBB",
];
const RIBBONS: [[&str; 4]; 4] = [
    ["....", "1...", ".133", ".13."],
    [".2..", "2.1.", "2.11", "...."],
    ["....", ".3..", "3322", "...2"],
    ["2..3", ".23.", ".233", "2..."],
];
const RUN_FEET: [[&str; 2]; 4] = [
    [".....B.B....B.B.", ".....B......B..."],
    [".....B.B....B.B.", "................"],
    [".....B.B....B.B.", ".......B......B."],
    [".....B.B....B.B.", "................"],
];
const RUN_EYES: [&str; 4] = [
    "....BBB..BBBB..B",
    "....BBBB..BBBB..",
    "....BBB..BBBB..B",
    "....BBBB..BBBB..",
];

fn run(k: usize) -> Pose {
    let rows: Vec<String> = (0..4)
        .map(|y| format!("{}{}", RIBBONS[k][y], &RUN_BODY[y][4..]))
        .chain([RUN_EYES[k].into()])
        .chain(RUN_BODY[5..].iter().map(|r| r.to_string()))
        .chain(RUN_FEET[k].map(String::from))
        .collect();
    pose(std::array::from_fn(|y| rows[y].as_str()))
}

/// Clawd squatting, down: the head drops a whole cell row, the eyes below
/// the bandana, the arms below the eyes and the feet squashed to one row,
/// while the ribbon is left floating above in one stripe colour. A drop of
/// one or two rows would put the bandana in the eyes cell row, a cell there
/// holding the bandana, the body and the empty eye
const SQUAT: [&str; 12] = [
    "................",
    "...............2",
    "...............2",
    "..............2.",
    "..BBBBBBBBBBBB..",
    "..BBBBBBBBBBBB..",
    "..112233mmmmmm..",
    "..112233mmmmmm..",
    "..BB..BBBB..BB..",
    "BBBBBBBBBBBBBBBB",
    "BBBBBBBBBBBBBBBB",
    "..B.B......B.B..",
];

/// halfway down: the head as he stands, as no drop short of a cell row fits
/// the rule, the arms a row lower, the ribbon drop a row longer to fill the
/// cell beside them, and the feet squashed to one row
const CROUCH: [&str; 12] = [
    "..BBBBBBBBBBBB..",
    "..BBBBBBBBBBBB..",
    "..112233mmmmmm3.",
    "..112233mmmmmm3.",
    "..BB..BBBB..BB22",
    "..BBBBBBBBBBBB22",
    "..BBBBBBBBBBBB22",
    "BBBBBBBBBBBBBBBB",
    "BBBBBBBBBBBBBBBB",
    "..BBBBBBBBBBBB..",
    "..BBBBBBBBBBBB..",
    "..B.B......B.B..",
];

#[derive(Clone, Copy)]
enum Side {
    Centre,
    Left,
    Right,
}

/// a step of Clawd waiting, held `frames` frames: standing, with his eyes, the
/// slits centre or a pixel left or right; his feet, down (`Centre`) or the
/// left or right pair lifted a row; and his left arm, 0 down, 1 half up, 2
/// up beside the eyes, 3 high beside the bandana. Each moves within cells of
/// body and empty alone; the right arm stays down, as above it the ribbon
/// shares its cells
#[derive(Clone, Copy)]
struct Wait {
    frames: usize,
    eyes: Side,
    feet: Side,
    arm: usize,
}

const STILL: Wait = Wait {
    frames: 1,
    eyes: Side::Centre,
    feet: Side::Centre,
    arm: 0,
};

impl Wait {
    fn pose(self) -> Pose {
        let mut rows = STAND.map(String::from);
        rows[4] = match self.eyes {
            Side::Centre => "..BB..BBBB..BB22",
            Side::Left => "..B..BBBB..BBB22",
            Side::Right => "..BBB..BBBB..B22",
        }
        .into();
        match self.feet {
            Side::Centre => {}
            Side::Left => rows[11] = "...........B.B..".into(),
            Side::Right => rows[11] = "..B.B...........".into(),
        }
        // the left arm, at rows 6 and 7 as he stands, moved; last, as the
        // eyes above set all of row 4
        for row in [6, 7] {
            rows[row].replace_range(..2, "..");
        }
        for row in [[6, 7], [5, 6], [4, 5], [2, 3]][self.arm] {
            rows[row].replace_range(..2, "BB");
        }
        pose(rows.each_ref().map(String::as_str))
    }
}

/// a wait, 6 s at 8 frames a second: looking left and right, a stamp, a
/// wave, then standing. The steps add up to 48
fn waiting() -> Vec<Pose> {
    let held = |frames| Wait { frames, ..STILL };
    let eyes = |frames, eyes| Wait {
        frames,
        eyes,
        ..STILL
    };
    let feet = |feet| Wait { feet, ..STILL };
    let arm = |arm| Wait { arm, ..STILL };
    [
        held(4),
        eyes(6, Side::Left),
        held(2),
        eyes(6, Side::Right),
        held(2),
        feet(Side::Left),
        held(2),
        feet(Side::Right),
        held(6),
        arm(1),
        arm(2),
        arm(3),
        arm(2),
        arm(3),
        arm(2),
        arm(3),
        arm(2),
        arm(1),
        held(9),
    ]
    .iter()
    .flat_map(|step| std::iter::repeat_n(step.pose(), step.frames))
    .collect()
}

/// a squat: down and back
fn squat_down() -> [Pose; 4] {
    [pose(CROUCH), pose(SQUAT), pose(SQUAT), pose(CROUCH)]
}

/// every pose Clawd takes, each once, and the animations as indices into
/// them, a pose a frame
struct Clawd {
    poses: Vec<Pose>,
    running: Vec<usize>,
    squatting: Vec<usize>,
    waiting: Vec<usize>,
    waiting_on_subagents: Vec<usize>,
    standing: Vec<usize>,
}

impl Clawd {
    fn new() -> Clawd {
        let mut poses = Vec::new();
        // an animation's poses as ids into `poses`, adding the new ones
        let mut pose_ids = |animation: Vec<Pose>| -> Vec<usize> {
            animation
                .into_iter()
                .map(|p| {
                    poses.iter().position(|q| *q == p).unwrap_or_else(|| {
                        poses.push(p);
                        poses.len() - 1
                    })
                })
                .collect()
        };
        let running = pose_ids((0..4).map(run).collect());
        // standing a while, then down and back
        let squatting = pose_ids(
            std::iter::repeat_n(pose(STAND), 8)
                .chain(squat_down())
                .collect(),
        );
        let waiting_poses = waiting();
        // a wait with a squat in each of its three longest rests, before the
        // look, between the stamp and the wave, and after the wave, about one
        // every 2 s: for a session waiting while its subagents work
        let mut on_subagents_poses = waiting_poses.clone();
        for at in [0, 25, 40] {
            on_subagents_poses[at..at + 4].copy_from_slice(&squat_down());
        }
        let waiting = pose_ids(waiting_poses);
        let waiting_on_subagents = pose_ids(on_subagents_poses);
        let standing = pose_ids(vec![pose(STAND)]);
        Clawd {
            poses,
            running,
            squatting,
            waiting,
            waiting_on_subagents,
            standing,
        }
    }

    /// the poses `session` cycles through, as ids into `poses`, a pose a
    /// frame, and its phase into them. Working, he runs, every runner in
    /// step; idle with subagents running, he squats;
    /// waiting, he looks about, stamps and waves, and squats too while
    /// subagents run; each squat and wait at its own phase from the session
    /// id; otherwise he stands
    fn animation(&self, session: &Session) -> (&[usize], usize) {
        let subagents_running = session.subagents_running();
        let phase = || djb2(&session.id) as usize;
        match session.state {
            State::Working => (&self.running, 0),
            State::Idle if subagents_running => (&self.squatting, phase()),
            State::Waiting if subagents_running => (&self.waiting_on_subagents, phase()),
            State::Waiting => (&self.waiting, phase()),
            State::Idle => (&self.standing, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_a_pinned_repo_by_its_pin_and_the_rest_around_it() {
        let session = |id: &str, repo: &str| {
            let text = format!(
                "{{\"session_id\":\"{id}\",\"tmux_pane\":\"%{id}\",\"repo\":\"{repo}\",\
                 \"state\":\"idle\",\"since\":1,\"subagents\":0}}"
            );
            Session::parse(&crate::json::parse_stream(&text).unwrap()[0]).unwrap()
        };
        let host = Host {
            label: "local".into(),
            ssh_host: String::new(),
            relay: String::new(),
            down: false,
            on_show: None,
            sessions: vec![
                session("1", "~"),
                session("2", "~/a"),
                session("3", "~/a"),
                session("4", "~/b"),
            ],
        };
        let view = View {
            width: 20,
            warn: 50.0,
            now: 0,
            pins: Pins::parse("1\t\t~/b\n"),
        };
        let fields: Vec<(String, String)> = render(&[host], &view, 1)[0]
            .split('\0')
            .skip(1)
            .filter(|item| !item.is_empty())
            .map(|item| {
                let f: Vec<&str> = item.split('\t').collect();
                (f[4].to_string(), f[6].to_string())
            })
            .collect();
        let expected = [("2", "~"), ("3", "~/a"), ("", ""), ("1", "~/b")];
        assert_eq!(
            fields,
            expected.map(|(n, r)| (n.to_string(), r.to_string()))
        );
    }

    #[test]
    fn every_pose_keeps_each_cell_to_two_colours() {
        let clawd = Clawd::new();
        for (i, pose) in clawd.poses.iter().enumerate() {
            for line in 0..3 {
                for cell in 0..8 {
                    let mut colours = cell_pixels(pose, line, cell).to_vec();
                    colours.sort();
                    colours.dedup();
                    assert!(
                        colours.len() <= 2,
                        "pose {i}, line {line}, cell {cell}: {:?}",
                        String::from_utf8_lossy(&colours)
                    );
                }
            }
        }
    }

    #[test]
    fn every_animation_loops_within_the_frames() {
        let clawd = Clawd::new();
        for animation in [
            &clawd.running,
            &clawd.squatting,
            &clawd.waiting,
            &clawd.waiting_on_subagents,
            &clawd.standing,
        ] {
            assert_eq!(FRAME_COUNT % animation.len(), 0);
        }
        assert_eq!(clawd.waiting.len(), 48);
    }

    #[test]
    fn the_octant_table_has_a_glyph_for_every_cell() {
        assert_eq!(OCTANTS.chars().count(), 256);
    }

    #[test]
    fn times_are_whole_minutes_or_hours() {
        let times: Vec<String> = [59.0, 60.0, 3599.0, 3600.0, 7300.0, -5.0]
            .into_iter()
            .map(ago)
            .collect();
        assert_eq!(times, ["<1m", "1m", "59m", "1h", "2h", "<1m"]);
    }

    #[test]
    fn colours_fade_and_turn_background_as_jq_does() {
        assert_eq!(faded(ORANGE), "38;2;140;92;69");
        assert_eq!(faded(YELLOW), "38;2;125;101;71");
        assert_eq!(as_background(YELLOW), "43");
        assert_eq!(as_background(DARK5), "48;2;115;122;162");
    }
}
