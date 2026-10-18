//! The sidebar's frames, rendered from the tagged sessions `claude-sidebar`
//! gathers: one NUL-terminated fzf item per header or session, the same
//! items in every frame but for Clawd's pose.
//!
//! A session's item is its tab fields: ssh host, pane, relay session,
//! session id (empty only on a header, which is how `jump` tells one), entry
//! number, 1 if the frame's right pane shows it, then its display lines.
//! Sessions with a pane are numbered in order across hosts, for `pick`.
//! Times are in whole minutes, so the rows change only when a session does,
//! or with the frame while a Clawd moves.

use crate::json::Value;

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

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// every frame of `tagged`'s items, frame 0 to `frames - 1`, each frame's
/// items one after another
pub fn render(tagged: &[Value], view: &View, frames: usize) -> Vec<String> {
    let clawd = Clawd::new();
    let mut out = vec![String::new(); frames];
    let mut number = 0;
    let mut on_show = Value::String(String::new());
    for r in tagged {
        if r.get("header").truthy() {
            on_show = r.get("current").clone();
            let item = header(r);
            for frame in &mut out {
                frame.push_str(&item);
            }
            continue;
        }
        let pane = r.get("tmux_pane");
        let numbered = pane != &Value::Null;
        if numbered {
            number += 1;
        }
        let entry = Entry::new(r, numbered.then_some(number), pane == &on_show, view);
        let (poses, phase) = clawd.animation(r);
        let palette = Palette::new(r);
        // each distinct pose drawn once, as most repeat across the frames
        // or, standing, are the same in all of them
        let mut drawn: Vec<Option<[String; 3]>> = vec![None; clawd.poses.len()];
        for (k, frame) in out.iter_mut().enumerate() {
            let pose = poses[(phase + k) % poses.len()];
            let picture =
                drawn[pose].get_or_insert_with(|| picture(&clawd.poses[pose], &palette, !numbered));
            entry.write(frame, picture);
        }
    }
    out
}

/// a host's heading: its label, `unreachable` if it did not answer, then a
/// blank second line, so the heading stands apart from the entries
fn header(r: &Value) -> String {
    let label = r.get("label").as_str().unwrap_or_default();
    // paint closes only the colour, so the reset ends the bold
    let mut item = format!("\t\t\t\t\t\t{}{RESET}", paint(label, &format!("1;{BLUE}")));
    if r.get("down").truthy() {
        item += &format!("  {}", paint("unreachable", RED));
    }
    // a braille blank (U+2800), since fzf drops a trailing line of spaces
    item + "\n\u{2800}\0"
}

/// a session's item, less the picture: what goes before and right of each
/// of its three lines
struct Entry {
    fields: String,
    /// dims each line of a session with no pane (a --bg one, or a daemon)
    dim: &'static str,
    texts: [String; 3],
}

impl Entry {
    fn new(r: &Value, number: Option<usize>, on_show: bool, view: &View) -> Entry {
        let pane = r.get("tmux_pane");
        let number_text = number.map(|n| n.to_string());
        let fields = [
            joined(r.get("host")),
            if pane.truthy() {
                joined(pane)
            } else {
                String::new()
            },
            joined(r.get("relay")),
            joined(r.get("session_id")),
            number_text.clone().unwrap_or_default(),
            if number.is_some() && on_show { "1" } else { "" }.to_string(),
            String::new(),
        ]
        .join("\t");
        let dim = if number.is_some() { "" } else { DIM };
        let text0 = format!(
            " {} {}\n",
            number_text.as_deref().unwrap_or(" "),
            last_part(r.get("cwd"))
        );
        let name = r.get("name");
        let text1 = format!(
            "   {}\n",
            if name.truthy() {
                name.text()
            } else {
                short_id(r.get("session_id"))
            }
        );
        let subagents = r.get("subagents");
        let mut text2 = format!(
            "   {} {}",
            r.get("state").text(),
            ago(view.now as f64 - r.get("since").as_f64().unwrap_or_default())
        );
        // running subagents in the working colour, so an idle session whose
        // subagents are still at work stands out
        if subagents.above_zero() {
            text2 += &format!("  {}", paint(&format!("+{}", subagents.text()), ORANGE));
        }
        if number.is_none() {
            text2 += &format!("  {dim}background{RESET}");
        }
        // reset at the end, or the dim of a background entry leaks into the
        // entries fzf draws after it
        text2 += &format!(
            "\n{dim}{}{RESET}",
            ctxline(r.get("context").get("used_pct"), view)
        );
        Entry {
            fields,
            dim,
            texts: [text0, text1, text2],
        }
    }

    /// appends the item, with `picture` left of its lines, to `frame`
    fn write(&self, frame: &mut String, picture: &[String; 3]) {
        frame.push_str(&self.fields);
        for (line, text) in picture.iter().zip(&self.texts) {
            frame.push_str(self.dim);
            frame.push_str(line);
            frame.push_str(text);
        }
        frame.push('\0');
    }
}

/// a field as jq's `join` takes it: null as empty, a string as itself
fn joined(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        v => v.text(),
    }
}

/// the last part of path `cwd`, as jq's `split("/") | last`, which has no
/// parts in an empty path
fn last_part(cwd: &Value) -> String {
    match cwd.as_str() {
        Some("") | None => "null".into(),
        Some(path) => path.rsplit('/').next().unwrap_or_default().into(),
    }
}

/// the first 8 characters of `id`
fn short_id(id: &Value) -> String {
    match id.as_str() {
        Some(id) => id.chars().take(8).collect(),
        None => id.text(),
    }
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

/// context use as a line across the entry, `view.width` columns with the
/// percentage label at its end: the used share a heavy rule, each cell
/// coloured by how far along the bar it sits, green to red as a full context
/// is the bad end. The rest a light dashed rule in terminal_black. All red at
/// `view.warn`. No record (`pct` null): the whole line terminal_black, with
/// no label
fn ctxline(pct: &Value, view: &View) -> String {
    let Some(share) = pct.as_f64() else {
        return paint(&"┈".repeat(view.width.max(0) as usize), TERMINAL_BLACK);
    };
    // padded to 4 columns, so bars up to 99% end in the same column
    let label = format!("{:>4}", format!(" {}%", pct.text()));
    let cells = view.width - label.chars().count() as i64;
    let full = (share * cells as f64 / 100.0).round().min(cells as f64);
    let warned = share >= view.warn;
    let palette: &[&str] = if warned {
        &[RED]
    } else {
        &[GREEN, YELLOW, ORANGE, RED]
    };
    let mut line = String::new();
    // painted a run of one colour at a time
    let mut run: Option<(&str, usize)> = None;
    for cell in 0..full.max(0.0) as i64 {
        let colour = palette[(cell as f64 * palette.len() as f64 / cells as f64).floor() as usize];
        match &mut run {
            Some((c, n)) if *c == colour => *n += 1,
            _ => {
                if let Some((c, n)) = run {
                    line += &paint(&"━".repeat(n), c);
                }
                run = Some((colour, 1));
            }
        }
    }
    if let Some((c, n)) = run {
        line += &paint(&"━".repeat(n), c);
    }
    let rest = (cells - full as i64).max(0) as usize;
    line += &paint(&"┈".repeat(rest), TERMINAL_BLACK);
    line + &if warned { paint(&label, RED) } else { label }
}

/// djb2 of a string, kept under 2^31 as jq, counting in doubles, kept it
fn djb(s: &str) -> u64 {
    s.chars()
        .fold(5381, |h, c| (h * 33 + c as u64) % 2147483647)
}

/// the colours Clawd is drawn in for a session, by the letters of a pose:
/// B his body, in the colour for its state (working orange, waiting yellow,
/// idle grey, the only states claude-session-state writes), m its bandana,
/// 1 2 3 the bandana's stripes
struct Palette {
    body: Option<&'static str>,
    /// main then stripes: four different colours from a stable hash of the
    /// session id, so a session wears the same bandana on every refresh and
    /// from either host. Each pick takes the hash modulo the colours left
    /// and passes the quotient on to the next
    bandana: [&'static str; 4],
}

impl Palette {
    fn new(r: &Value) -> Palette {
        let body = match r.get("state").as_str() {
            Some("working") => Some(ORANGE),
            Some("waiting") => Some(YELLOW),
            Some("idle") => Some(DARK5),
            _ => None,
        };
        let mut hash = djb(r.get("session_id").as_str().unwrap_or_default());
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
            b'B' => self.body,
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
                let pixels = std::array::from_fn(|k| {
                    palette.colour(pose[4 * line + k / 2][2 * cell + k % 2])
                });
                octant(pixels, faint)
            })
            .collect()
    })
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
/// not drawn. Faint dims only a foreground, so where `faint`, a cell with a
/// background turns faint off (22) and draws both colours faded, then turns
/// faint back on
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
    match background {
        None => paint(&glyph, ink),
        Some(bg) if faint => format!("\x1b[22m{}{DIM}", paint_on(&glyph, &faded(ink), &faded(bg))),
        Some(bg) => paint_on(&glyph, ink, bg),
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
    let ribbon: Vec<String> = (0..4)
        .map(|y| format!("{}{}", RIBBONS[k][y], &RUN_BODY[y][4..]))
        .collect();
    pose([
        &ribbon[0],
        &ribbon[1],
        &ribbon[2],
        &ribbon[3],
        RUN_EYES[k],
        RUN_BODY[5],
        RUN_BODY[6],
        RUN_BODY[7],
        RUN_BODY[8],
        RUN_BODY[9],
        RUN_FEET[k][0],
        RUN_FEET[k][1],
    ])
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

/// a step of Clawd waiting, held `n` frames: standing, with his eyes, the
/// slits centre or a pixel left or right; his feet, down (`Centre`) or the
/// left or right pair lifted a row; and his left arm, 0 down, 1 half up, 2
/// up beside the eyes, 3 high beside the bandana. Each moves within cells of
/// body and empty alone; the right arm stays down, as above it the ribbon
/// shares its cells
#[derive(Clone, Copy)]
struct Wait {
    n: usize,
    eyes: Side,
    feet: Side,
    arm: usize,
}

const STILL: Wait = Wait {
    n: 1,
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
    let held = |n| Wait { n, ..STILL };
    let eyes = |n, eyes| Wait { n, eyes, ..STILL };
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
    .flat_map(|step| std::iter::repeat_n(step.pose(), step.n))
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
        let mut ids = |animation: Vec<Pose>| -> Vec<usize> {
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
        let running = ids((0..4).map(run).collect());
        // standing a while, then down and back
        let squatting = ids(std::iter::repeat_n(pose(STAND), 8)
            .chain(squat_down())
            .collect());
        let waiting_poses = waiting();
        // a wait with a squat in each of its three longest rests, before the
        // look, between the stamp and the wave, and after the wave, about one
        // every 2 s: for a session waiting while its subagents work
        let mut on_subagents = waiting_poses.clone();
        for at in [0, 25, 40] {
            on_subagents[at..at + 4].copy_from_slice(&squat_down());
        }
        let waiting = ids(waiting_poses);
        let waiting_on_subagents = ids(on_subagents);
        let standing = ids(vec![pose(STAND)]);
        Clawd {
            poses,
            running,
            squatting,
            waiting,
            waiting_on_subagents,
            standing,
        }
    }

    /// the poses session `r` cycles through, a pose a frame, and its phase
    /// into them. Working, he runs; idle with subagents running, he squats;
    /// waiting, he looks about, stamps and waves, and squats too while
    /// subagents run; each squat and wait at its own phase from the session
    /// id; otherwise he stands
    fn animation(&self, r: &Value) -> (&[usize], usize) {
        let state = r.get("state").as_str();
        let busy = r.get("subagents").above_zero();
        let phase = || djb(r.get("session_id").as_str().unwrap_or_default()) as usize;
        match state {
            Some("working") => (&self.running, 0),
            Some("idle") if busy => (&self.squatting, phase()),
            Some("waiting") if busy => (&self.waiting_on_subagents, phase()),
            Some("waiting") => (&self.waiting, phase()),
            _ => (&self.standing, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pose_keeps_each_cell_to_two_colours() {
        let clawd = Clawd::new();
        for (i, pose) in clawd.poses.iter().enumerate() {
            for line in 0..3 {
                for cell in 0..8 {
                    let mut colours: Vec<u8> = (0..8)
                        .map(|k| pose[4 * line + k / 2][2 * cell + k % 2])
                        .collect();
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
