use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::ExitCode;

use indexmap::IndexMap;
use rgb::RGB8;
use zipng::v2::{Encoder, FontChoice, PaletteChoice};

const MAX_FILE_SIZE: usize = 60 * 1024; // 60KB limit

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Create,
    Extract,
    List,
}

fn print_help(default_mode: Mode) {
    let prog = if default_mode == Mode::Extract {
        "unzipng"
    } else {
        "zipng"
    };
    let default_label = match default_mode {
        Mode::Create => " (default)",
        _ => "",
    };
    let extract_label = match default_mode {
        Mode::Extract => " (default)",
        _ => "",
    };
    eprintln!(
        "Usage: {prog} [mode] [options] [archive] [files...]

Polyglot PNG+ZIP file tool.

Modes:
  -c, --create       Create a new archive{default_label}
  -x, --extract      Extract files from an archive{extract_label}
  -t, -l, --list     List archive contents

Options:
  -f, --file FILE    Archive path (overrides 1st positional)
      --in FILE      Repeatable. Archive (extract/list) or files (create)
  -o, --out PATH     Output archive (create) or directory (extract)
  -C, -d, --directory DIR  Alias for --out in extract mode
  -n, --no-clobber   No-op (default: don't overwrite)
      --force         Overwrite existing files
  -v, --verbose      Increase verbosity
  -q, --quiet        Suppress output
  -@                 Read file list from stdin
      --colors SPEC  Palette name (e.g. viridis, magma) or hex colors
                     (e.g. ffffff,ff0000,000000)
      --sort         Sort palette colors (default)
      --no-sort      Don't sort palette colors
      --font NAME    Font: swiss, sixth, sky, monte, sugimori, mini, micro
  -h, --help         Show this help

Positional arguments:
  create:  <archive> <files...>
  extract: <archive> [files to extract...]
  list:    <archive> [files to list...]

Examples:
  {prog} -o archive.png file1.txt file2.txt
  {prog} archive.png src/lib.rs src/v2.rs
  {prog} --colors viridis -o archive.png *.rs
  {prog} --colors ffffff,3b82f6,000000 archive.png file.txt
  find . -name '*.rs' | {prog} -@ -o archive.png"
    );
}

fn parse_font(name: &str) -> Result<FontChoice, String> {
    match name.to_ascii_lowercase().as_str() {
        "swiss" => Ok(FontChoice::Swiss),
        "sixth" => Ok(FontChoice::Sixth),
        "sky" => Ok(FontChoice::Sky),
        "monte" => Ok(FontChoice::Monte),
        "sugimori" => Ok(FontChoice::Sugimori),
        "mini" => Ok(FontChoice::Mini),
        "micro" => Ok(FontChoice::Micro),
        _ => Err(format!(
            "unknown font '{}' (options: swiss, sixth, sky, monte, sugimori, mini, micro)",
            name
        )),
    }
}

fn parse_colors_spec(s: &str) -> Result<PaletteChoice, String> {
    // First try as a named palette (case-insensitive)
    if let Some(palette) = lookup_named_palette(s) {
        return Ok(PaletteChoice::Custom(palette.to_vec()));
    }
    // Fall back to comma-separated hex parsing
    let colors = parse_hex_colors(s)?;
    let rgb_arrays: Vec<[u8; 3]> = colors.iter().map(|c| [c.r, c.g, c.b]).collect();
    Ok(PaletteChoice::Colors(rgb_arrays))
}

fn lookup_named_palette(name: &str) -> Option<&'static [u8]> {
    use zipng::palettes::*;
    let lower = name.to_ascii_lowercase();
    let lower = lower.replace('-', "_");
    match lower.as_str() {
        // viridis family
        "viridis" => Some(viridis::VIRIDIS),
        "magma" => Some(viridis::MAGMA),
        "inferno" => Some(viridis::INFERNO),
        "plasma" => Some(viridis::PLASMA),
        // oceanic sequential
        "algae" => Some(oceanic::ALGAE),
        "amp" => Some(oceanic::AMP),
        "deep" => Some(oceanic::DEEP),
        "dense" => Some(oceanic::DENSE),
        "haline" => Some(oceanic::HALINE),
        "ice" => Some(oceanic::ICE),
        "matter" => Some(oceanic::MATTER),
        "oxy" => Some(oceanic::OXY),
        "rain" => Some(oceanic::RAIN),
        "solar" => Some(oceanic::SOLAR),
        "speed" => Some(oceanic::SPEED),
        "tempo" => Some(oceanic::TEMPO),
        "thermal" => Some(oceanic::THERMAL),
        "turbid" => Some(oceanic::TURBID),
        // oceanic diverging
        "balance" => Some(oceanic::BALANCE),
        "curl" => Some(oceanic::CURL),
        "delta" => Some(oceanic::DELTA),
        "diff" => Some(oceanic::DIFF),
        "tarn" => Some(oceanic::TARN),
        // oceanic other
        "topo" => Some(oceanic::TOPO),
        "phase" => Some(oceanic::PHASE),
        "gray" => Some(oceanic::GRAY),
        // crameri sequential
        "acton" => Some(crameri::ACTON),
        "bamako" => Some(crameri::BAMAKO),
        "batlow" => Some(crameri::BATLOW),
        "batlow_k" | "batlowk" => Some(crameri::BATLOW_K),
        "batlow_w" | "batloww" => Some(crameri::BATLOW_W),
        "bilbao" => Some(crameri::BILBAO),
        "buda" => Some(crameri::BUDA),
        "davos" => Some(crameri::DAVOS),
        "devon" => Some(crameri::DEVON),
        "hawaii" => Some(crameri::HAWAII),
        "imola" => Some(crameri::IMOLA),
        "lajolla" => Some(crameri::LAJOLLA),
        "lapaz" => Some(crameri::LAPAZ),
        "nuuk" => Some(crameri::NUUK),
        "oslo" => Some(crameri::OSLO),
        "tokyo" => Some(crameri::TOKYO),
        "turku" => Some(crameri::TURKU),
        // crameri diverging
        "bam" => Some(crameri::BAM),
        "berlin" => Some(crameri::BERLIN),
        "broc" => Some(crameri::BROC),
        "cork" => Some(crameri::CORK),
        "lisbon" => Some(crameri::LISBON),
        "roma" => Some(crameri::ROMA),
        "tofino" => Some(crameri::TOFINO),
        "vanimo" => Some(crameri::VANIMO),
        "vik" => Some(crameri::VIK),
        // crameri cyclic
        "bam_o" | "bamo" => Some(crameri::BAM_O),
        "broc_o" | "broco" => Some(crameri::BROC_O),
        "cork_o" | "corko" => Some(crameri::CORK_O),
        "roma_o" | "romao" => Some(crameri::ROMA_O),
        "vik_o" | "viko" => Some(crameri::VIK_O),
        // crameri dual-sequential
        "bukavu" => Some(crameri::BUKAVU),
        "fes" => Some(crameri::FES),
        "oleron" => Some(crameri::OLERON),
        // crameri grayscale
        "gray_c" | "grayc" => Some(crameri::GRAY_C),
        // singles
        "turbo" => Some(singles::TURBO),
        "cividis" => Some(singles::CIVIDIS),
        _ => None,
    }
}

fn parse_hex_colors(s: &str) -> Result<Vec<RGB8>, String> {
    let colors: Result<Vec<RGB8>, String> = s
        .split(',')
        .map(|hex| {
            let hex = hex.trim().trim_start_matches('#');
            let (r, g, b) = if hex.len() == 3 {
                let r = u8::from_str_radix(&hex[0..1], 16)
                    .map_err(|_| format!("invalid hex color '{}'", hex))?;
                let g = u8::from_str_radix(&hex[1..2], 16)
                    .map_err(|_| format!("invalid hex color '{}'", hex))?;
                let b = u8::from_str_radix(&hex[2..3], 16)
                    .map_err(|_| format!("invalid hex color '{}'", hex))?;
                (r << 4 | r, g << 4 | g, b << 4 | b)
            } else if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| format!("invalid hex color '{}'", hex))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| format!("invalid hex color '{}'", hex))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| format!("invalid hex color '{}'", hex))?;
                (r, g, b)
            } else {
                return Err(format!(
                    "invalid hex color '{}': expected 3 or 6 hex digits",
                    hex
                ));
            };
            Ok(RGB8::new(r, g, b))
        })
        .collect();
    let colors = colors?;
    if colors.is_empty() {
        return Err("--colors requires at least one color".to_string());
    }
    Ok(colors)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let binary_name = args
        .first()
        .and_then(|a| Path::new(a).file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let default_mode = if binary_name.contains("unzipng") {
        Mode::Extract
    } else {
        Mode::Create
    };

    let args: Vec<String> = args.into_iter().skip(1).collect();

    if args.is_empty() {
        print_help(default_mode);
        return ExitCode::from(1);
    }

    // Parse arguments
    let mut mode: Option<Mode> = None;
    let mut file_flag: Option<String> = None;
    let mut in_args: Vec<String> = Vec::new();
    let mut out_path: Option<String> = None;
    let mut force = false;
    let mut verbose: u32 = 0;
    let mut quiet = false;
    let mut read_stdin = false;
    let mut colors_spec: Option<String> = None;
    let mut sort_colors = true; // default: sort
    let mut font_choice: Option<FontChoice> = None;
    let mut positionals: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help(default_mode);
                return ExitCode::SUCCESS;
            }
            "-c" | "--create" => {
                mode = Some(Mode::Create);
                i += 1;
            }
            "-x" | "--extract" => {
                mode = Some(Mode::Extract);
                i += 1;
            }
            "-t" | "-l" | "--list" => {
                mode = Some(Mode::List);
                i += 1;
            }
            "-f" | "--file" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: {} requires an argument", args[i]);
                    return ExitCode::from(1);
                }
                file_flag = Some(args[i + 1].clone());
                i += 2;
            }
            "--in" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --in requires an argument");
                    return ExitCode::from(1);
                }
                in_args.push(args[i + 1].clone());
                i += 2;
            }
            "-o" | "--out" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: {} requires an argument", args[i]);
                    return ExitCode::from(1);
                }
                out_path = Some(args[i + 1].clone());
                i += 2;
            }
            "-C" | "-d" | "--directory" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: {} requires an argument", args[i]);
                    return ExitCode::from(1);
                }
                out_path = Some(args[i + 1].clone());
                i += 2;
            }
            "-n" | "--no-clobber" => {
                // no-op, default behavior
                i += 1;
            }
            "--force" => {
                force = true;
                i += 1;
            }
            "-v" | "--verbose" => {
                verbose += 1;
                i += 1;
            }
            "-q" | "--quiet" => {
                quiet = true;
                i += 1;
            }
            "-@" => {
                read_stdin = true;
                i += 1;
            }
            "--colors" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --colors requires an argument");
                    return ExitCode::from(1);
                }
                colors_spec = Some(args[i + 1].clone());
                i += 2;
            }
            "--sort" => {
                sort_colors = true;
                i += 1;
            }
            "--no-sort" => {
                sort_colors = false;
                i += 1;
            }
            "--font" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --font requires an argument");
                    return ExitCode::from(1);
                }
                match parse_font(&args[i + 1]) {
                    Ok(f) => font_choice = Some(f),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return ExitCode::from(1);
                    }
                }
                i += 2;
            }
            arg if arg.starts_with('-') && arg != "-" => {
                eprintln!("Error: unknown option: {}", arg);
                return ExitCode::from(1);
            }
            _ => {
                positionals.push(args[i].clone());
                i += 1;
            }
        }
    }

    let mode = mode.unwrap_or(default_mode);

    // Read stdin file list if -@
    let mut stdin_files: Vec<String> = Vec::new();
    if read_stdin {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(path) => {
                    let path = path.trim().to_string();
                    if !path.is_empty() {
                        stdin_files.push(path);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    return ExitCode::from(1);
                }
            }
        }
    }

    match mode {
        Mode::Create => run_create(
            file_flag,
            in_args,
            out_path,
            positionals,
            stdin_files,
            force,
            verbose,
            quiet,
            colors_spec,
            sort_colors,
            font_choice,
        ),
        Mode::Extract => run_extract(file_flag, in_args, out_path, positionals, force, verbose, quiet),
        Mode::List => run_list(file_flag, in_args, positionals, verbose, quiet),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_create(
    file_flag: Option<String>,
    in_args: Vec<String>,
    out_path: Option<String>,
    positionals: Vec<String>,
    stdin_files: Vec<String>,
    force: bool,
    _verbose: u32,
    quiet: bool,
    colors_spec: Option<String>,
    sort_colors: bool,
    font_choice: Option<FontChoice>,
) -> ExitCode {
    // Determine output path: --out, or -f, or first positional
    let has_explicit_out = out_path.is_some() || file_flag.is_some();
    let archive_path = out_path
        .or_else(|| file_flag.clone())
        .or_else(|| positionals.first().cloned());

    let archive_path = match archive_path {
        Some(p) => p,
        None => {
            eprintln!("Error: no output path specified (use --out or provide as first argument)");
            return ExitCode::from(1);
        }
    };

    // Check overwrite
    if !force && Path::new(&archive_path).exists() {
        eprintln!(
            "Error: '{}' already exists (use --force to overwrite)",
            archive_path
        );
        return ExitCode::from(1);
    }

    // Collect input files: remaining positionals (skip 1st if it was the archive),
    // plus --in values, plus stdin files
    let first_pos_is_archive = !has_explicit_out;
    let pos_inputs: Vec<String> = if first_pos_is_archive {
        positionals.into_iter().skip(1).collect()
    } else {
        positionals
    };

    let mut input_files: Vec<String> = Vec::new();
    input_files.extend(pos_inputs);
    input_files.extend(in_args);
    input_files.extend(stdin_files);

    if input_files.is_empty() {
        eprintln!("Error: no input files specified");
        return ExitCode::from(1);
    }

    // Build file map
    let mut files: IndexMap<Vec<u8>, Vec<u8>> = IndexMap::new();

    for path in &input_files {
        let file_path = Path::new(path);

        if !file_path.exists() {
            eprintln!("Error: file not found: {}", path);
            return ExitCode::from(1);
        }

        let contents = match fs::read(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", path, e);
                return ExitCode::from(1);
            }
        };

        if contents.len() > MAX_FILE_SIZE {
            eprintln!(
                "Error: {} exceeds 60KB limit ({:.1} KB)",
                path,
                contents.len() as f64 / 1024.0
            );
            return ExitCode::from(1);
        }

        files.insert(path.as_bytes().to_vec(), contents);
    }

    // Build encoder
    let mut encoder = Encoder::new();

    if let Some(spec) = colors_spec {
        match parse_colors_spec(&spec) {
            Ok(mut palette) => {
                // For PaletteChoice::Colors, sort is handled by the encoder.
                // For Custom (named palette), if --no-sort we pass as-is.
                // The v2 Colors variant always sorts internally, so if --no-sort
                // with hex colors, we use Custom instead.
                if !sort_colors {
                    if let PaletteChoice::Colors(ref colors) = palette {
                        // Convert to a raw palette without sorting
                        let rgb_colors: Vec<RGB8> = colors
                            .iter()
                            .map(|c| RGB8::new(c[0], c[1], c[2]))
                            .collect();
                        let raw = zipng::palettes::perceptual::generate(&rgb_colors);
                        palette = PaletteChoice::Custom(raw);
                    }
                }
                encoder = encoder.with_palette(palette);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return ExitCode::from(1);
            }
        }
    }

    if let Some(font) = font_choice {
        encoder = encoder.with_font(font);
    }

    // Generate polyglot PNG+ZIP
    let output = encoder.encode(files.iter().map(|(k, v)| (k.as_slice(), v.as_slice())));

    // Write output
    if let Err(e) = fs::write(&archive_path, &output) {
        eprintln!("Error writing {}: {}", archive_path, e);
        return ExitCode::from(1);
    }

    if !quiet {
        let size_kib = output.len() as f64 / 1024.0;
        let file_count = input_files.len();
        let file_word = if file_count == 1 { "file" } else { "files" };
        println!(
            "Created {} ({} {}, {:.1} KiB)",
            archive_path, file_count, file_word, size_kib
        );
    }

    ExitCode::SUCCESS
}

fn run_extract(
    _file_flag: Option<String>,
    _in_args: Vec<String>,
    _out_path: Option<String>,
    _positionals: Vec<String>,
    _force: bool,
    _verbose: u32,
    _quiet: bool,
) -> ExitCode {
    unimplemented!("zipng extract mode is not yet implemented");
}

fn run_list(
    _file_flag: Option<String>,
    _in_args: Vec<String>,
    _positionals: Vec<String>,
    _verbose: u32,
    _quiet: bool,
) -> ExitCode {
    unimplemented!("zipng list mode is not yet implemented");
}
