use std::str::FromStr;

/// テキスト内の要素。
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    /// 通常テキスト。
    Text(String),
    /// 色変更（`<#...>`）。
    Color {
        code: ColorType,
    },
    /// 文字サイズ変更（`<s...>`）。
    Size {
        size: ScalarValue,
        font: Option<String>,
        bold: bool,
        italic: bool,
        strikethrough: bool,
        outline_size: Option<f64>,
    },
    /// フォント変更（`<@...>`）。
    Font {
        name: String,
    },
    /// 表示速度変更（`<r...>`）。
    Speed {
        speed: Option<f64>,
    },
    /// 表示の一時停止（`<w...>`）。
    Wait {
        time: TimeValue,
    },
    /// 表示のクリア（`<c...>`）。
    Clear {
        time: TimeValue,
    },
    /// 位置変更（`<p...>`）。
    Position {
        x: AxisValue,
        y: AxisValue,
        z: Option<AxisValue>,
    },
    /// スクリプト実行（`<?...?>`）。
    Script {
        code: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorType {
    Default,
    Single(ColorValue),
    Pair(ColorValue, ColorValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorValue {
    Rgb(u8, u8, u8),
    Preset(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScalarValue {
    Default,
    Absolute(f64),
    RelativeAdd(f64),
    RelativeMul(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeValue {
    Default,
    Absolute(f64),
    PerChar(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AxisValue {
    Absolute(f64),
    Relative(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Elements(pub Vec<Element>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseElementError;

impl std::fmt::Display for ParseElementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to parse Element from string")
    }
}

impl std::error::Error for ParseElementError {}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(value) => write!(f, "{}", value),
            Self::Color { code } => write!(f, "<#{}>", code),
            Self::Size { size, font, bold, italic, strikethrough, outline_size } => {
                write!(f, "<s{}", size)?;
                let has_decoration = *bold || *italic || *strikethrough;
                if font.is_some() || has_decoration || outline_size.is_some() {
                    write!(f, ",{}", font.as_deref().unwrap_or(""))?;
                    if has_decoration || outline_size.is_some() {
                        let mut deco = String::new();
                        if *bold { deco.push('B'); }
                        if *italic { deco.push('I'); }
                        if *strikethrough { deco.push('S'); }
                        write!(f, ",{}", deco)?;
                        if let Some(outline) = outline_size {
                            write!(f, ",{}", trim_float(*outline))?;
                        }
                    }
                }
                write!(f, ">")
            }
            Self::Font { name } => write!(f, "<@{}>", name),
            Self::Speed { speed } => match speed {
                Some(speed) => write!(f, "<r{}>", trim_float(*speed)),
                None => write!(f, "<r>"),
            },
            Self::Wait { time } => write!(f, "<w{}>", time),
            Self::Clear { time } => write!(f, "<c{}>", time),
            Self::Position { x, y, z } => {
                write!(f, "<p{},{}", x, y)?;
                if let Some(z) = z {
                    write!(f, ",{}", z)?;
                }
                write!(f, ">")
            }
            Self::Script { code } => write!(f, "<?{}?>", code),
        }
    }
}

impl std::fmt::Display for ColorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => Ok(()),
            Self::Single(value) => write!(f, "{}", value),
            Self::Pair(a, b) => write!(f, "{},{}", a, b),
        }
    }
}

impl std::fmt::Display for ColorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rgb(r, g, b) => write!(f, "{:02x}{:02x}{:02x}", r, g, b),
            Self::Preset(name) => write!(f, "{}", name),
        }
    }
}

impl std::fmt::Display for ScalarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => Ok(()),
            Self::Absolute(v) => write!(f, "{}", trim_float(*v)),
            Self::RelativeAdd(v) => write!(f, "{}", format_signed(*v)),
            Self::RelativeMul(v) => write!(f, "*{}", trim_float(*v)),
        }
    }
}

impl std::fmt::Display for TimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => Ok(()),
            Self::Absolute(v) => write!(f, "{}", trim_float(*v)),
            Self::PerChar(v) => write!(f, "*{}", trim_float(*v)),
        }
    }
}

impl std::fmt::Display for AxisValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absolute(v) => write!(f, "{}", trim_float(*v)),
            Self::Relative(v) => write!(f, "{}", format_signed(*v)),
        }
    }
}

impl std::fmt::Display for Elements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for element in &self.0 {
            write!(f, "{}", element)?;
        }
        Ok(())
    }
}

impl FromStr for Element {
    type Err = ParseElementError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let char_len = s.chars().count();
        if let Some((next, consumed)) = parse_control_sequence_str(s) {
            if consumed == char_len {
                return Ok(parse_control_element(next, s.to_string()));
            }
            return Err(ParseElementError);
        }

        Ok(Self::Text(s.to_string()))
    }
}

impl FromStr for Elements {
    type Err = ParseElementError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(parse_text(s)))
    }
}

/// Processes escape sequences in the input string.
///
/// - `\\` → `\`
/// - `\n` → newline
/// - Any other `\x` → `\` (backslash is kept, single char advance)
pub fn parse_escape(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut out = String::new();

    while i < len {
        let c = chars[i];
        if c == '\\' {
            match chars.get(i + 1).copied() {
                Some('\\') => {
                    out.push('\\');
                    i += 2;
                }
                Some('n') => {
                    out.push('\n');
                    i += 2;
                }
                Some(c) => {
                    out.push('\\');
                    out.push(c);
                    i += 2;
                }
                None => {
                    out.push('\\');
                    i += 1;
                }
            }
        } else {
            out.push(c);
            i += 1;
        }
    }

    out
}

/// Parses control sequences (e.g. `<#ffffff>`, `<s32>`) from the input string.
///
/// Unrecognized `<...>` sequences are treated as literal text. Does not
/// process escape sequences — run [`parse_escape`] first if needed.
pub fn parse_control(text: &str) -> Vec<Element> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut text_buffer = String::new();
    let mut out = Vec::new();

    while i < len {
        let c = chars[i];
        if c != '<' {
            text_buffer.push(c);
            i += 1;
            continue;
        }

        let consumed = parse_control_sequence(&chars, i);
        if consumed == 0 {
            text_buffer.push('<');
            i += 1;
        } else {
            if !text_buffer.is_empty() {
                out.push(Element::Text(std::mem::take(&mut text_buffer)));
            }
            let next = chars[i + 1];
            let control_sequence: String = chars[i..i + consumed].iter().collect();
            out.push(parse_control_element(next, control_sequence));
            i += consumed;
        }
    }

    if !text_buffer.is_empty() {
        out.push(Element::Text(text_buffer));
    }

    out
}

/// Parses a text string that may contain both escape sequences and control
/// sequences. Equivalent to `parse_control(&parse_escape(text))`.
pub fn parse_text(text: &str) -> Vec<Element> {
    parse_control(&parse_escape(text))
}

/// Maps a logical character index (skipping `\n` and `\t` separators) to the
/// corresponding byte index in `text`.
pub fn object_index_to_string_index(text: &str, index: usize) -> Option<usize> {
    let mut count = 0;
    for (i, c) in text.char_indices() {
        if c == '\n' || c == '\t' {
            continue;
        }
        if count == index {
            return Some(i);
        }
        count += 1;
    }
    None
}

fn consume_c_tag(input: &str) -> usize {
    lazy_regex::regex_find!(r"^<c(?:\*?[0-9.]+)?>", input)
        .map(|matched| matched.chars().count())
        .unwrap_or(0)
}

fn consume_script_tag(input: &str) -> usize {
    lazy_regex::regex_find!(r"^<\?=?[\s\S]*?>", input)
        .map(|matched| matched.chars().count())
        .unwrap_or(0)
}

fn parse_control_sequence(chars: &[char], i: usize) -> usize {
    let Some(next) = chars.get(i + 1).copied() else {
        return 0;
    };
    let rest: String = chars[i..].iter().collect();

    match next {
        '#' | '@' => consume_color_or_font_tag(&rest),
        's' => consume_s_tag(&rest),
        'r' => consume_r_tag(&rest),
        'w' => consume_w_tag(&rest),
        'c' => consume_c_tag(&rest),
        'p' => consume_p_tag(&rest),
        '?' => consume_script_tag(&rest),
        _ => 0,
    }
}

fn parse_control_element(next: char, control_sequence: String) -> Element {
    let body = match next {
        '?' => &control_sequence[2..control_sequence.len() - 2],
        _ => &control_sequence[2..control_sequence.len() - 1],
    };
    match next {
        '#' => Element::Color {
            code: parse_color_type(body),
        },
        's' => parse_size_element(body),
        '@' => Element::Font {
            name: body.to_string(),
        },
        'r' => Element::Speed {
            speed: parse_optional_f64(body),
        },
        'w' => Element::Wait {
            time: parse_time_value(body),
        },
        'c' => Element::Clear {
            time: parse_time_value(body),
        },
        'p' => parse_position_element(body),
        '?' => Element::Script {
            code: body.to_string(),
        },
        _ => Element::Text(control_sequence),
    }
}

fn parse_color_type(body: &str) -> ColorType {
    if body.is_empty() {
        return ColorType::Default;
    }
    let mut split = body.splitn(2, ',');
    let first = split.next().unwrap_or_default();
    let second = split.next();
    match second {
        Some(second) => ColorType::Pair(parse_color_value(first), parse_color_value(second)),
        None => ColorType::Single(parse_color_value(first)),
    }
}

fn parse_color_value(token: &str) -> ColorValue {
    if token.len() == 6 && token.chars().all(|c| c.is_ascii_hexdigit()) {
        let r = u8::from_str_radix(&token[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&token[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&token[4..6], 16).unwrap_or(0);
        ColorValue::Rgb(r, g, b)
    } else {
        ColorValue::Preset(token.to_string())
    }
}

fn parse_scalar_value(token: &str) -> ScalarValue {
    if token.is_empty() {
        return ScalarValue::Default;
    }
    if let Some(value) = token.strip_prefix('*') {
        return ScalarValue::RelativeMul(value.parse::<f64>().unwrap_or(0.0));
    }
    if token.starts_with('+') || token.starts_with('-') {
        return ScalarValue::RelativeAdd(token.parse::<f64>().unwrap_or(0.0));
    }
    ScalarValue::Absolute(token.parse::<f64>().unwrap_or(0.0))
}

fn parse_optional_f64(token: &str) -> Option<f64> {
    if token.is_empty() {
        None
    } else {
        Some(token.parse::<f64>().unwrap_or(0.0))
    }
}

fn parse_time_value(token: &str) -> TimeValue {
    if token.is_empty() {
        return TimeValue::Default;
    }
    if let Some(value) = token.strip_prefix('*') {
        return TimeValue::PerChar(value.parse::<f64>().unwrap_or(0.0));
    }
    TimeValue::Absolute(token.parse::<f64>().unwrap_or(0.0))
}

fn parse_axis_value(token: &str) -> AxisValue {
    if token.starts_with('+') || token.starts_with('-') {
        AxisValue::Relative(token.parse::<f64>().unwrap_or(0.0))
    } else {
        AxisValue::Absolute(token.parse::<f64>().unwrap_or(0.0))
    }
}

fn parse_size_element(body: &str) -> Element {
    let mut parts = body.splitn(4, ',');
    let size = parse_scalar_value(parts.next().unwrap_or_default().trim());
    let font = parts
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let decoration = parts.next().unwrap_or_default();
    let bold = decoration.contains('B');
    let italic = decoration.contains('I');
    let strikethrough = decoration.contains('S');
    let outline_size = parts.next().and_then(|s| s.trim().parse::<f64>().ok());
    Element::Size { size, font, bold, italic, strikethrough, outline_size }
}

fn parse_position_element(body: &str) -> Element {
    let mut values = body.split(',');
    let x = parse_axis_value(values.next().unwrap_or("0").trim());
    let y = parse_axis_value(values.next().unwrap_or("0").trim());
    let z = values.next().map(|v| parse_axis_value(v.trim()));
    Element::Position { x, y, z }
}

fn format_signed(value: f64) -> String {
    if value >= 0.0 {
        format!("+{}", trim_float(value))
    } else {
        trim_float(value)
    }
}

fn trim_float(value: f64) -> String {
    let normalized = if value == -0.0 { 0.0 } else { value };
    let mut s = normalized.to_string();
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

fn parse_control_sequence_str(input: &str) -> Option<(char, usize)> {
    let mut chars = input.chars();
    if chars.next()? != '<' {
        return None;
    }
    let next = chars.next()?;
    let consumed = match next {
        '#' | '@' => consume_color_or_font_tag(input),
        's' => consume_s_tag(input),
        'r' => consume_r_tag(input),
        'w' => consume_w_tag(input),
        'c' => consume_c_tag(input),
        'p' => consume_p_tag(input),
        '?' => consume_script_tag(input),
        _ => 0,
    };
    if consumed > 0 {
        Some((next, consumed))
    } else {
        None
    }
}

fn consume_color_or_font_tag(input: &str) -> usize {
    lazy_regex::regex_find!(r"^<[#@][^>]*>", input)
        .map(|matched| matched.chars().count())
        .unwrap_or(0)
}

fn consume_s_tag(input: &str) -> usize {
    lazy_regex::regex_find!(
        r"^<s(?:[+-][0-9.]+|\*[0-9.]+|[0-9.]*)(?:,[^,]*(?:,[BIS]*(?:,[0-9.]+)?)?)?>",
        input
    )
    .map(|matched| matched.chars().count())
    .unwrap_or(0)
}

fn consume_r_tag(input: &str) -> usize {
    lazy_regex::regex_find!(r"^<r[0-9.]*>", input)
        .map(|matched| matched.chars().count())
        .unwrap_or(0)
}

fn consume_w_tag(input: &str) -> usize {
    lazy_regex::regex_find!(r"^<w\*?[0-9.]+>", input)
        .map(|matched| matched.chars().count())
        .unwrap_or(0)
}

fn consume_p_tag(input: &str) -> usize {
    lazy_regex::regex_find!(r"^<p[+-]?[0-9.]+,[+-]?[0-9.]+(?:,[+-]?[0-9.]+)?>", input)
        .map(|matched| matched.chars().count())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_escape ---

    #[test]
    fn test_escape_newline() {
        assert_eq!(parse_escape(r"\n"), "\n");
    }

    #[test]
    fn test_escape_backslash() {
        assert_eq!(parse_escape(r"\\"), "\\");
    }

    #[test]
    fn test_escape_unknown_kept() {
        assert_eq!(parse_escape(r"\t"), r"\t");
    }

    #[test]
    fn test_escape_mixed() {
        assert_eq!(parse_escape(r"Hello\nWorld\\!"), "Hello\nWorld\\!");
    }

    // --- parse_control: color ---

    #[test]
    fn test_color_rgb() {
        let result = parse_control("<#ffffff>");
        assert_eq!(
            result,
            vec![Element::Color {
                code: ColorType::Single(ColorValue::Rgb(255, 255, 255))
            }]
        );
    }

    #[test]
    fn test_color_pair() {
        let result = parse_control("<#000000,ffffff>");
        assert_eq!(
            result,
            vec![Element::Color {
                code: ColorType::Pair(ColorValue::Rgb(0, 0, 0), ColorValue::Rgb(255, 255, 255))
            }]
        );
    }

    #[test]
    fn test_color_default() {
        let result = parse_control("<#>");
        assert_eq!(
            result,
            vec![Element::Color {
                code: ColorType::Default
            }]
        );
    }

    #[test]
    fn test_color_preset() {
        let result = parse_control("<#red>");
        assert_eq!(
            result,
            vec![Element::Color {
                code: ColorType::Single(ColorValue::Preset("red".to_string()))
            }]
        );
    }

    // --- parse_control: size ---

    #[test]
    fn test_size_absolute() {
        let result = parse_control("<s32>");
        assert_eq!(
            result,
            vec![Element::Size {
                size: ScalarValue::Absolute(32.0),
                font: None,
                bold: false,
                italic: false,
                strikethrough: false,
                outline_size: None,
            }]
        );
    }

    #[test]
    fn test_size_default() {
        let result = parse_control("<s>");
        assert_eq!(
            result,
            vec![Element::Size {
                size: ScalarValue::Default,
                font: None,
                bold: false,
                italic: false,
                strikethrough: false,
                outline_size: None,
            }]
        );
    }

    #[test]
    fn test_size_with_font() {
        let result = parse_control("<s72,メイリオ>");
        assert_eq!(
            result,
            vec![Element::Size {
                size: ScalarValue::Absolute(72.0),
                font: Some("メイリオ".to_string()),
                bold: false,
                italic: false,
                strikethrough: false,
                outline_size: None,
            }]
        );
    }

    #[test]
    fn test_size_with_font_and_decoration() {
        let result = parse_control("<s72,メイリオ,BI>");
        assert_eq!(
            result,
            vec![Element::Size {
                size: ScalarValue::Absolute(72.0),
                font: Some("メイリオ".to_string()),
                bold: true,
                italic: true,
                strikethrough: false,
                outline_size: None,
            }]
        );
    }

    #[test]
    fn test_size_with_outline() {
        let result = parse_control("<s72,メイリオ,BI,4>");
        assert_eq!(
            result,
            vec![Element::Size {
                size: ScalarValue::Absolute(72.0),
                font: Some("メイリオ".to_string()),
                bold: true,
                italic: true,
                strikethrough: false,
                outline_size: Some(4.0),
            }]
        );
    }

    #[test]
    fn test_size_decoration_no_font() {
        let result = parse_control("<s32,,S>");
        assert_eq!(
            result,
            vec![Element::Size {
                size: ScalarValue::Absolute(32.0),
                font: None,
                bold: false,
                italic: false,
                strikethrough: true,
                outline_size: None,
            }]
        );
    }

    #[test]
    fn test_size_relative_add() {
        let result = parse_control("<s+10>");
        assert_eq!(
            result,
            vec![Element::Size {
                size: ScalarValue::RelativeAdd(10.0),
                font: None,
                bold: false,
                italic: false,
                strikethrough: false,
                outline_size: None,
            }]
        );
    }

    #[test]
    fn test_size_relative_mul() {
        let result = parse_control("<s*1.5>");
        assert_eq!(
            result,
            vec![Element::Size {
                size: ScalarValue::RelativeMul(1.5),
                font: None,
                bold: false,
                italic: false,
                strikethrough: false,
                outline_size: None,
            }]
        );
    }

    // --- parse_control: font ---

    #[test]
    fn test_font() {
        let result = parse_control("<@メイリオ>");
        assert_eq!(
            result,
            vec![Element::Font {
                name: "メイリオ".to_string()
            }]
        );
    }

    // --- parse_control: speed ---

    #[test]
    fn test_speed_value() {
        let result = parse_control("<r10>");
        assert_eq!(result, vec![Element::Speed { speed: Some(10.0) }]);
    }

    #[test]
    fn test_speed_default() {
        let result = parse_control("<r>");
        assert_eq!(result, vec![Element::Speed { speed: None }]);
    }

    // --- parse_control: wait ---

    #[test]
    fn test_wait_absolute() {
        let result = parse_control("<w5>");
        assert_eq!(
            result,
            vec![Element::Wait {
                time: TimeValue::Absolute(5.0)
            }]
        );
    }

    #[test]
    fn test_wait_fractional() {
        let result = parse_control("<w0.5>");
        assert_eq!(
            result,
            vec![Element::Wait {
                time: TimeValue::Absolute(0.5)
            }]
        );
    }

    #[test]
    fn test_wait_per_char() {
        let result = parse_control("<w*0.2>");
        assert_eq!(
            result,
            vec![Element::Wait {
                time: TimeValue::PerChar(0.2)
            }]
        );
    }

    // --- parse_control: clear ---

    #[test]
    fn test_clear_default() {
        let result = parse_control("<c>");
        assert_eq!(
            result,
            vec![Element::Clear {
                time: TimeValue::Default
            }]
        );
    }

    #[test]
    fn test_clear_absolute() {
        let result = parse_control("<c5>");
        assert_eq!(
            result,
            vec![Element::Clear {
                time: TimeValue::Absolute(5.0)
            }]
        );
    }

    #[test]
    fn test_clear_per_char() {
        let result = parse_control("<c*0.2>");
        assert_eq!(
            result,
            vec![Element::Clear {
                time: TimeValue::PerChar(0.2)
            }]
        );
    }

    // --- parse_control: position ---

    #[test]
    fn test_position_2d() {
        let result = parse_control("<p20,40>");
        assert_eq!(
            result,
            vec![Element::Position {
                x: AxisValue::Absolute(20.0),
                y: AxisValue::Absolute(40.0),
                z: None,
            }]
        );
    }

    #[test]
    fn test_position_3d() {
        let result = parse_control("<p20,40,80>");
        assert_eq!(
            result,
            vec![Element::Position {
                x: AxisValue::Absolute(20.0),
                y: AxisValue::Absolute(40.0),
                z: Some(AxisValue::Absolute(80.0)),
            }]
        );
    }

    #[test]
    fn test_position_relative() {
        let result = parse_control("<p+10,+10>");
        assert_eq!(
            result,
            vec![Element::Position {
                x: AxisValue::Relative(10.0),
                y: AxisValue::Relative(10.0),
                z: None,
            }]
        );
    }

    // --- parse_control: script ---

    #[test]
    fn test_script() {
        let result = parse_control("<?obj.rz=obj.time*360?>");
        assert_eq!(
            result,
            vec![Element::Script {
                code: "obj.rz=obj.time*360".to_string()
            }]
        );
    }

    #[test]
    fn test_script_expression() {
        let result = parse_control(r#"<?=string.format("%02d:%02d",obj.time/60,obj.time%60)?>"#);
        assert_eq!(
            result,
            vec![Element::Script {
                code: r#"=string.format("%02d:%02d",obj.time/60,obj.time%60)"#.to_string()
            }]
        );
    }

    // --- parse_text (escape + control combined) ---

    #[test]
    fn test_parse_text() {
        let input = r"Hello\nWorld<#FF0000><s1,2,BI><r0.5><w*1.5><p10,-5>Red";
        let expected = vec![
            Element::Text("Hello\nWorld".to_string()),
            Element::Color {
                code: ColorType::Single(ColorValue::Rgb(255, 0, 0)),
            },
            Element::Size {
                size: ScalarValue::Absolute(1.0),
                font: Some("2".to_string()),
                bold: true,
                italic: true,
                strikethrough: false,
                outline_size: None,
            },
            Element::Speed { speed: Some(0.5) },
            Element::Wait {
                time: TimeValue::PerChar(1.5),
            },
            Element::Position {
                x: AxisValue::Absolute(10.0),
                y: AxisValue::Relative(-5.0),
                z: None,
            },
            Element::Text("Red".to_string()),
        ];
        let result = parse_text(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_text_all_control_sequences() {
        let input = "<#red><s32><@Meiryo><r10><w0.5><c5><p+1,-2><?obj.rz=1?>";
        let expected = vec![
            Element::Color {
                code: ColorType::Single(ColorValue::Preset("red".to_string())),
            },
            Element::Size {
                size: ScalarValue::Absolute(32.0),
                font: None,
                bold: false,
                italic: false,
                strikethrough: false,
                outline_size: None,
            },
            Element::Font {
                name: "Meiryo".to_string(),
            },
            Element::Speed { speed: Some(10.0) },
            Element::Wait {
                time: TimeValue::Absolute(0.5),
            },
            Element::Clear {
                time: TimeValue::Absolute(5.0),
            },
            Element::Position {
                x: AxisValue::Relative(1.0),
                y: AxisValue::Relative(-2.0),
                z: None,
            },
            Element::Script {
                code: "obj.rz=1".to_string(),
            },
        ];
        let result = parse_text(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_element_to_string_roundtrip() {
        let input = r"Hello\n<#ffffff><@Meiryo>World";
        let parsed = parse_text(input);
        let restored = parsed
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("");
        assert_eq!(restored, "Hello\n<#ffffff><@Meiryo>World");
    }

    #[test]
    fn test_element_from_str_text() {
        let parsed = "Hello".parse::<Element>().unwrap();
        assert_eq!(parsed, Element::Text("Hello".to_string()));
    }

    #[test]
    fn test_element_from_str_control_sequence() {
        let parsed = "<w0.5>".parse::<Element>().unwrap();
        assert_eq!(
            parsed,
            Element::Wait {
                time: TimeValue::Absolute(0.5)
            }
        );
    }

    #[test]
    fn test_element_from_str_rejects_mixed_input() {
        let parsed = "<w0.5>text".parse::<Element>();
        assert!(parsed.is_err());
    }

    #[test]
    fn test_elements_from_str() {
        let parsed = "Hi<w1>!".parse::<Elements>().unwrap();
        assert_eq!(
            parsed,
            Elements(vec![
                Element::Text("Hi".to_string()),
                Element::Wait {
                    time: TimeValue::Absolute(1.0),
                },
                Element::Text("!".to_string()),
            ])
        );
    }

    #[test]
    fn test_elements_to_string_roundtrip() {
        let input = r"A\nB<@Meiryo>C";
        let parsed = input.parse::<Elements>().unwrap();
        assert_eq!(parsed.to_string(), "A\nB<@Meiryo>C");
    }

    #[test]
    fn test_object_index_to_string_index() {
        let text = "A\nB\tC";
        assert_eq!(object_index_to_string_index(text, 0), Some(0)); // 'A'
        assert_eq!(object_index_to_string_index(text, 1), Some(2)); // 'B'
        assert_eq!(object_index_to_string_index(text, 2), Some(4)); // 'C'
        assert_eq!(object_index_to_string_index(text, 3), None); // Out of bounds
    }
}
