// Copyright (c) 2026 rezky_nightky

//! CLI argument definitions and help output generators.
//!
//! Cosmostrix follows a **curated simplicity** philosophy:
//! - `--help` shows only the most common, user-facing options
//! - `--help-detail` provides the full engineering reference
//! - Advanced tuning knobs (glitch, shading, linger, etc.) are hidden from
//!   the first impression but remain fully functional for power users.

use std::io::IsTerminal;
use std::str::FromStr;

use clap::Parser;

#[must_use]
pub fn color_enabled_stdout() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if matches!(std::env::var("CLICOLOR").ok().as_deref(), Some("0")) {
        return false;
    }
    std::io::stdout().is_terminal()
}

fn colorize_help_detail(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 64);
    for chunk in text.split_inclusive('\n') {
        let (line, nl) = chunk
            .strip_suffix('\n')
            .map(|l| (l, "\n"))
            .unwrap_or((chunk, ""));

        let is_heading =
            !line.starts_with(' ') && line.ends_with(':') && line == line.to_ascii_uppercase();

        if is_heading {
            out.push_str("\x1b[1;36m");
            out.push_str(line);
            out.push_str("\x1b[0m");
            out.push_str(nl);
            continue;
        }

        if let Some(rest) = line.strip_prefix("      Example:") {
            out.push_str("      \x1b[32mExample:\x1b[0m");
            out.push_str(rest);
            out.push_str(nl);
            continue;
        }

        if let Some(rest) = line.strip_prefix("  cosmostrix") {
            out.push_str("  \x1b[1;34mcosmostrix\x1b[0m");
            out.push_str(rest);
            out.push_str(nl);
            continue;
        }

        if let Some(rest) = line.strip_prefix("  -") {
            out.push_str("  \x1b[33m-");
            out.push_str(rest);
            out.push_str("\x1b[0m");
            out.push_str(nl);
            continue;
        }

        if let Some(rest) = line.strip_prefix("  --") {
            out.push_str("  \x1b[33m--");
            out.push_str(rest);
            out.push_str("\x1b[0m");
            out.push_str(nl);
            continue;
        }

        out.push_str(line);
        out.push_str(nl);
    }
    out
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorBg {
    #[value(name = "black")]
    Black,
    #[value(name = "default-background")]
    DefaultBackground,
    #[value(name = "transparent")]
    Transparent,
}

#[derive(Clone, Copy, Debug)]
pub struct U16Range {
    pub low: u16,
    pub high: u16,
}

impl FromStr for U16Range {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (a, b) = s
            .split_once(',')
            .ok_or_else(|| "expected: NUM1,NUM2".to_string())?;
        let low: u16 = a
            .trim()
            .parse()
            .map_err(|_| "invalid low value".to_string())?;
        let high: u16 = b
            .trim()
            .parse()
            .map_err(|_| "invalid high value".to_string())?;
        if low == 0 || high == 0 || low > high {
            return Err("range must be >0 and low <= high (min allowed value is 1)".to_string());
        }
        Ok(Self { low, high })
    }
}

// ---------------------------------------------------------------------------
// Args — curated two-tier help design
//
// VISIBLE args appear in --help (the first impression).
// HIDDEN args are still fully functional but only documented in --help-detail.
// ---------------------------------------------------------------------------

#[derive(Parser, Debug, Clone)]
#[command(
    name = "cosmostrix",
    version,
    disable_version_flag = true,
    about = "High-performance cinematic terminal renderer"
)]
pub struct Args {
    // === COMMON OPTIONS (visible in --help) ===
    #[arg(
        short = 'c',
        long = "color",
        default_value = "green",
        help_heading = "COMMON OPTIONS",
        display_order = 10,
        help = "Color theme (see --list-colors)"
    )]
    pub color: String,

    #[arg(
        long = "charset",
        default_value = "binary",
        help_heading = "COMMON OPTIONS",
        display_order = 20,
        help = "Character preset (see --list-charsets)"
    )]
    pub charset: String,

    #[arg(
        short = 'f',
        long = "fps",
        default_value_t = 60.0,
        help_heading = "COMMON OPTIONS",
        display_order = 30,
        help = "Target FPS"
    )]
    pub fps: f64,

    #[arg(
        short = 'S',
        long = "speed",
        default_value_t = 8.0,
        help_heading = "COMMON OPTIONS",
        display_order = 40,
        help = "Rain speed"
    )]
    pub speed: f32,

    #[arg(
        short = 'd',
        long = "density",
        default_value_t = 1.0,
        help_heading = "COMMON OPTIONS",
        display_order = 50,
        help = "Rain density"
    )]
    pub density: f32,

    #[arg(
        short = 's',
        long = "screensaver",
        help_heading = "COMMON OPTIONS",
        display_order = 60,
        help = "Screensaver mode (exit on keypress)"
    )]
    pub screensaver: bool,

    #[arg(
        short = 'm',
        long = "message",
        help_heading = "COMMON OPTIONS",
        display_order = 70,
        help = "Overlay message"
    )]
    pub message: Option<String>,

    #[arg(
        long = "low-power",
        help_heading = "COMMON OPTIONS",
        display_order = 80,
        help = "Power-saving mode (30 FPS, reduced density/speed)"
    )]
    pub low_power: bool,

    // === DIAGNOSTICS (visible in --help) ===
    #[arg(
        long = "doctor",
        help_heading = "DIAGNOSTICS",
        display_order = 100,
        help = "System compatibility report"
    )]
    pub doctor: bool,

    #[arg(
        long = "benchmark",
        help_heading = "DIAGNOSTICS",
        display_order = 110,
        help = "Renderer benchmark"
    )]
    pub benchmark: bool,

    #[arg(
        long = "info",
        short = 'i',
        help_heading = "DIAGNOSTICS",
        display_order = 120,
        help = "Build and runtime information"
    )]
    pub info: bool,

    // === DISCOVERY (visible in --help) ===
    #[arg(
        long = "list-colors",
        help_heading = "DISCOVERY",
        display_order = 200,
        help = "Show available color themes"
    )]
    pub list_colors: bool,

    #[arg(
        long = "list-charsets",
        help_heading = "DISCOVERY",
        display_order = 210,
        help = "Show available charset presets"
    )]
    pub list_charsets: bool,

    // === HELP (visible in --help) ===
    #[arg(
        long = "help-detail",
        help_heading = "HELP",
        display_order = 300,
        help = "Full advanced documentation"
    )]
    pub help_detail: bool,

    #[arg(
        long = "version",
        short = 'v',
        help_heading = "HELP",
        display_order = 320,
        help = "Show version"
    )]
    pub version: bool,

    // === HIDDEN (advanced — documented in --help-detail) ===
    #[arg(
        short = 'a',
        long = "async",
        default_value_t = false,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        hide = true,
        help = "Async rendering (default: off)"
    )]
    pub async_mode: bool,

    #[arg(
        short = 'b',
        long = "bold",
        default_value_t = 1,
        hide = true,
        help = "Bold style: 0=off, 1=random, 2=all (min 0 max 2)"
    )]
    pub bold: u8,

    #[arg(
        long = "color-bg",
        default_value_t = ColorBg::Black,
        value_enum,
        hide = true,
        help = "Background mode (black, default-background, transparent)"
    )]
    pub color_bg: ColorBg,

    #[arg(
        short = 'F',
        long = "fullwidth",
        hide = true,
        help = "Use full terminal width"
    )]
    pub fullwidth: bool,

    #[arg(
        long = "duration",
        hide = true,
        help = "Stop after N seconds (min 0.1 max 86400; <=0 disables)"
    )]
    pub duration: Option<f64>,

    #[arg(
        long = "perf-stats",
        hide = true,
        help = "Print performance statistics summary on exit"
    )]
    pub perf_stats: bool,

    #[arg(
        long = "bench-frames",
        hide = true,
        help = "Run headless benchmark for N frames and exit"
    )]
    pub bench_frames: Option<u64>,

    #[arg(
        short = 'g',
        long = "glitchms",
        default_value = "300,400",
        hide = true,
        help = "Glitch duration range in ms: LOW,HIGH (min 1 max 5000)"
    )]
    pub glitch_ms: U16Range,

    #[arg(
        short = 'G',
        long = "glitchpct",
        default_value_t = 10.0,
        hide = true,
        help = "Glitch chance in percent (min 0 max 100)"
    )]
    pub glitch_pct: f32,

    #[arg(
        short = 'l',
        long = "lingerms",
        default_value = "1,3000",
        hide = true,
        help = "Linger time range in ms: LOW,HIGH (min 1 max 60000)"
    )]
    pub linger_ms: U16Range,

    #[arg(
        short = 'M',
        long = "shadingmode",
        default_value_t = 1,
        hide = true,
        help = "Shading: 0=random, 1=distance-from-head (min 0 max 1)"
    )]
    pub shading_mode: u8,

    #[arg(
        long = "message-no-border",
        hide = true,
        help = "Draw message box without border (use with --message; shorthand: -mB)"
    )]
    pub message_no_border: bool,

    #[arg(
        long = "maxdpc",
        default_value_t = 3,
        hide = true,
        help = "Max droplets per column (min 1 max 3)"
    )]
    pub max_droplets_per_column: u8,

    #[arg(
        long = "noglitch",
        default_value_t = true,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        hide = true,
        help = "Disable glitch effects (default: on)"
    )]
    pub noglitch: bool,

    #[arg(
        short = 'r',
        long = "rippct",
        default_value_t = 33.33333,
        hide = true,
        help = "Die-early chance in percent (min 0 max 100)"
    )]
    pub rippct: f32,

    #[arg(
        long = "shortpct",
        default_value_t = 50.0,
        hide = true,
        help = "Chance for short droplets in percent (min 0 max 100)"
    )]
    pub shortpct: f32,

    #[arg(long = "chars", hide = true, help = "Custom characters override")]
    pub chars: Option<String>,

    #[arg(
        long = "colormode",
        hide = true,
        help = "Force color mode (allowed: 0,16,8/256,24/32). Default: 24-bit if supported (COLORTERM), else 8-bit (TERM=...256color), else 16-color"
    )]
    pub colormode: Option<u16>,

    #[arg(
        long = "check-bitcolor",
        hide = true,
        help = "Print detected terminal color capability and exit"
    )]
    pub check_bitcolor: bool,
}

// ---------------------------------------------------------------------------
// List printers
// ---------------------------------------------------------------------------

pub fn print_list_charsets() {
    if color_enabled_stdout() {
        println!("\x1b[1;36mAVAILABLE CHARSET PRESETS:\x1b[0m");
        println!("\x1b[2mNOTE: Use only the VALUE (left side) with --charset.\x1b[0m");
    } else {
        println!("AVAILABLE CHARSET PRESETS:");
        println!("NOTE: Use only the VALUE (left side) with --charset.");
    }
    println!();
    println!("VALUE        DESCRIPTION");
    println!("auto         Auto-select (ASCII_SAFE when non-UTF, otherwise matrix)");
    println!("matrix       Letters + digits + katakana (no punctuation)");
    println!("ascii        Letters + digits + punctuation");
    println!("extended     Digits + punctuation + katakana");
    println!("english      Letters only");
    println!("digits       Digits only (aliases: dec, decimal)");
    println!("punc         Punctuation only");
    println!("binary       0 and 1 (aliases: bin, 01)");
    println!("hex          0-9 and A-F (alias: hexadecimal)");
    println!("katakana     Katakana");
    println!("greek        Greek");
    println!("cyrillic     Cyrillic");
    println!("hebrew       Hebrew");
    println!("blocks       Block elements (shading blocks)");
    println!("symbols      Math/technical symbols");
    println!("arrows       Arrow symbols");
    println!("retro        Box-drawing characters");
    println!("cyberpunk    Katakana + hex + symbols (combo)");
    println!("hacker       Letters + hex + punc + symbols (combo)");
    println!("minimal      Dots and simple shapes");
    println!("code         Letters + digits + punc + symbols (combo)");
    println!("dna          DNA bases (ACGT)");
    println!("braille      Braille");
    println!("runic        Runic");
}

pub fn print_list_colors() {
    if color_enabled_stdout() {
        println!("\x1b[1;36mAVAILABLE COLOR THEMES:\x1b[0m");
        println!("\x1b[2mNOTE: Use only the VALUE (left side) with --color.\x1b[0m");
    } else {
        println!("AVAILABLE COLOR THEMES:");
        println!("NOTE: Use only the VALUE (left side) with --color.");
    }
    println!();
    println!("VALUE        DESCRIPTION");
    println!("green        Green theme");
    println!("green2       Green variant");
    println!("green3       Green variant");
    println!("yellow       Yellow theme");
    println!("orange       Orange theme");
    println!("red          Red theme");
    println!("blue         Blue theme");
    println!("cyan         Cyan theme");
    println!("gold         Gold theme");
    println!("rainbow      Rainbow theme");
    println!("purple       Purple theme");
    println!("neon         Neon theme (alias: synthwave)");
    println!("fire         Fire theme (alias: inferno)");
    println!("ocean        Ocean theme (alias: deep-sea)");
    println!("forest       Forest theme (alias: jungle)");
    println!("vaporwave    Vaporwave theme");
    println!("spectrum20   Spectrum 20-color theme (aliases: theme20, spectrum-20)");
    println!("gray         Gray theme (alias: grey)");
    println!("snow         Snow / ice theme");
    println!("aurora       Aurora theme");
    println!("fancy-diamond Fancy diamond theme");
    println!("cosmos       Cosmos theme");
    println!("nebula       Nebula theme");
    println!("stars        Stars theme");
    println!("mars         Mars theme");
    println!("venus        Venus theme");
    println!("mercury      Mercury theme");
    println!("jupiter      Jupiter theme");
    println!("saturn       Saturn theme");
    println!("uranus       Uranus theme");
    println!("neptune      Neptune theme");
    println!("pluto        Pluto theme");
    println!("moon         Moon theme");
    println!("sun          Sun theme");
    println!("comet        Comet theme");
    println!("galaxy       Galaxy theme");
    println!("supernova    Supernova theme");
    println!("blackhole    Black hole theme");
    println!("andromeda    Andromeda theme");
    println!("stardust     Stardust theme");
    println!("meteor       Meteor theme");
    println!("eclipse      Eclipse theme");
    println!("deepspace    Deep space theme");
}

// ---------------------------------------------------------------------------
// --help-detail: full engineering reference
// ---------------------------------------------------------------------------

pub fn print_help_detail() {
    let common = "USAGE:
  cosmostrix [OPTIONS]

COMMON OPTIONS:
  -c, --color <name>
      Set theme (see --list-colors).
      Example: cosmostrix --color rainbow

  --charset <name>
      Charset preset (see --list-charsets).
      Example: cosmostrix --charset binary

  -f, --fps <number>
      Target FPS (min 1 max 240) [default: 60].
      Example: cosmostrix --fps 30

  -S, --speed <number>
      Characters per second (rain speed) (min 0.001 max 1000) [default: 8].
      Example: cosmostrix --speed 12

  -d, --density <number>
      Droplet density (min 0.01 max 5.0) [default: 1.0].
      Example: cosmostrix --density 1.25

  -s, --screensaver
      Screensaver mode (exit on keypress).
      Example: cosmostrix -s

  -m, --message <text>
      Overlay message.
      Example: cosmostrix -m \"hello\"

  --low-power
      Power-saving mode. Overrides default values for FPS, speed, and density
      when those flags are not explicitly set:
        - FPS: 30 (if not explicitly set)
        - Speed: 5 (if not explicitly set)
        - Density: 0.5 (if not explicitly set)
      Explicit CLI flags always take precedence over --low-power defaults.
      Example: cosmostrix --low-power
      Example: cosmostrix --low-power --fps 24

APPEARANCE (ADVANCED):
  --colormode <0|8|24>
      Force color mode; otherwise auto-detected from COLORTERM/TERM.
      Example: cosmostrix --colormode 24

  -b, --bold <0|1|2>
      Bold style (0 off, 1 random, 2 all) [default: 1].
      Example: cosmostrix --bold 2

  -M, --shadingmode <0|1>
      Shading (0 random, 1 distance-from-head) [default: 1].
      Example: cosmostrix -M 1

  --color-bg <black|default-background|transparent>
      Background mode.
      Example: cosmostrix --color-bg transparent

GENERAL (ADVANCED):
  -a, --async
      Async rendering (default: off).
      To enable: --async or --async=true
      Example: cosmostrix --async

  -F, --fullwidth
      Use full terminal width.
      Example: cosmostrix -F

  --duration <seconds>
      Stop after N seconds (min 0.1 max 86400).
      Example: cosmostrix --duration 10

  --message-no-border, -mB
      Draw filled box without border characters.

PERFORMANCE (ADVANCED):
  --maxdpc <number>
      Max droplets per column (min 1 max 3) [default: 3].
      Example: cosmostrix --maxdpc 2

  --perf-stats
      Print performance statistics summary on exit.
      Example: cosmostrix --duration 10 --perf-stats

CHARSET (ADVANCED):
  --chars <string>
      Custom character override.
      Example: cosmostrix --chars \"01\"

GLITCH (ADVANCED):
  --noglitch
      Disable glitch effects (default: on).
      To enable glitch: --noglitch=false
      Example: cosmostrix --noglitch=false

  -G, --glitchpct <number>
      Glitch chance in percent (min 0 max 100) [default: 10].
      Example: cosmostrix --glitchpct 5

  -g, --glitchms <low,high>
      Glitch duration range in ms (min 1 max 5000) [default: 300,400].
      Example: cosmostrix --glitchms 200,500

  -l, --lingerms <low,high>
      Linger duration range in ms (min 1 max 60000) [default: 1,3000].
      Example: cosmostrix --lingerms 1,3000

  --shortpct <number>
      Short droplet chance in percent (min 0 max 100) [default: 50].
      Example: cosmostrix --shortpct 40

  -r, --rippct <number>
      Die-early chance in percent (min 0 max 100) [default: 33.33333].
      Example: cosmostrix --rippct 20

DIAGNOSTICS:
  --doctor
      Print compatibility report and exit.
      Example: cosmostrix --doctor

  --benchmark
      Run renderer benchmark (5 seconds) and print results.
      Example: cosmostrix --benchmark

  --check-bitcolor
      Print detected terminal color capability and exit.
      Example: cosmostrix --check-bitcolor

  --bench-frames <frames>
      Run headless benchmark for N frames and exit.
      Example: cosmostrix --fps 60 --bench-frames 200000

HELP:
  --help
      Show short help (curated common options only).

  --help-detail
      Show this detailed help (full engineering reference).

  --list-charsets
      List available charset presets and exit.

  --list-colors
      List available color themes and exit.

  -v, --version
      Print version and exit.

  -i, --info
      Print version info and exit.
";

    if color_enabled_stdout() {
        print!("{}", colorize_help_detail(common));
    } else {
        print!("{}", common);
    }

    let runtime_keys = "RUNTIME KEYS:
  q / Esc
      Quit
  p
      Pause/resume
  Ctrl+Z
      Suspend (resume with: fg)
  Space
      Reset/reseed animation
  Up / Down
      Increase/decrease speed
  [ / -
      Decrease density
  ] / +
      Increase density
  c / C
      Cycle color theme (next/previous)
  s / S
      Cycle charset preset (next/previous)
  a
      Toggle async rendering
  g
      Toggle glitch effects on/off
  m
      Cycle behavior profile (Monolith/Void/Neural/Decay/Eclipse/Static/Pulse)
  Left / Right
      Change glitch percent (when glitch is on)
  Tab
      Toggle shading mode
";
    if color_enabled_stdout() {
        print!("{}", colorize_help_detail(runtime_keys));
    } else {
        print!("{}", runtime_keys);
    }

    let env = "ENVIRONMENT:
  COSMOSTRIX_NO_FORK_GUARD
      Linux only. Set to 1/true/on/yes to disable the fork-based SIGKILL (-9) terminal guard.
      Values 0/false/off/no/empty keep the guard enabled.
";
    if color_enabled_stdout() {
        print!("{}", colorize_help_detail(env));
    } else {
        print!("{}", env);
    }

    let tail = "VALUE LISTS:
  cosmostrix --list-charsets
  cosmostrix --list-colors

LIMITS / VALID RANGES:
";
    if color_enabled_stdout() {
        print!("{}", colorize_help_detail(tail));
    } else {
        print!("{}", tail);
    }
    println!("  --duration <seconds>     min 0.1 max 86400 (<=0 disables)");
    println!("  --perf-stats             print performance summary on exit");
    println!("  --bench-frames <frames>  min 1");
    println!("  --fps <number>           min 1 max 240 [default: 60]");
    println!("  --speed <number>         min 0.001 max 1000 [default: 8]");
    println!("  --density <number>       min 0.01 max 5.0 [default: 1.0]");
    println!("  --maxdpc <number>        min 1 max 3 [default: 3]");
    println!("  --glitchpct <number>     min 0 max 100 [default: 10]");
    println!("  --shortpct <number>      min 0 max 100 [default: 50]");
    println!("  --rippct <number>        min 0 max 100 [default: 33.33333]");
    println!("  --glitchms <low,high>    min 1 max 5000 (each) [default: 300,400]");
    println!("  --lingerms <low,high>    min 1 max 60000 (each) [default: 1,3000]");
    println!("  --bold <0|1|2>           min 0 max 2 [default: 1]");
    println!("  --shadingmode <0|1>      min 0 max 1 [default: 1]");
    println!("  --colormode <0|16|8|24>  allowed values only (8==256, 24==32)");
    println!();
    print_list_charsets();
    println!();
    print_list_colors();
}
