/// Renders every built-in palette and several perceptual colormaps as 256×64
/// horizontal gradient PNG images into `target/palette-gallery/`.

use image::{ImageBuffer, Rgb};
use rgb::RGB8;
use std::fs;
use std::path::Path;

use zipng::palettes;
use zipng::palettes::perceptual;

fn main() {
    let out_dir = Path::new("target/palette-gallery");
    fs::create_dir_all(out_dir).unwrap();

    // ── Built-in palettes ──────────────────────────────────────────────

    let builtin: Vec<(&str, &[u8])> = vec![
        // Sequential
        ("seq-amp", palettes::oceanic::AMP),
        ("seq-ice", palettes::oceanic::ICE),
        ("seq-oxy", palettes::oceanic::OXY),
        ("seq-buda", palettes::crameri::BUDA),
        ("seq-nuuk", palettes::crameri::NUUK),
        ("seq-oslo", palettes::crameri::OSLO),
        ("seq-deep", palettes::oceanic::DEEP),
        ("seq-rain", palettes::oceanic::RAIN),
        ("seq-acton", palettes::crameri::ACTON),
        ("seq-davos", palettes::crameri::DAVOS),
        ("seq-devon", palettes::crameri::DEVON),
        ("seq-imola", palettes::crameri::IMOLA),
        ("seq-lapaz", palettes::crameri::LAPAZ),
        ("seq-tokyo", palettes::crameri::TOKYO),
        ("seq-turku", palettes::crameri::TURKU),
        ("seq-algae", palettes::oceanic::ALGAE),
        ("seq-dense", palettes::oceanic::DENSE),
        ("seq-solar", palettes::oceanic::SOLAR),
        ("seq-speed", palettes::oceanic::SPEED),
        ("seq-tempo", palettes::oceanic::TEMPO),
        ("seq-turbo", palettes::singles::TURBO),
        ("seq-magma", palettes::viridis::MAGMA),
        ("seq-bamako", palettes::crameri::BAMAKO),
        ("seq-batlow", palettes::crameri::BATLOW),
        ("seq-bilbao", palettes::crameri::BILBAO),
        ("seq-hawaii", palettes::crameri::HAWAII),
        ("seq-haline", palettes::oceanic::HALINE),
        ("seq-matter", palettes::oceanic::MATTER),
        ("seq-turbid", palettes::oceanic::TURBID),
        ("seq-plasma", palettes::viridis::PLASMA),
        ("seq-lajolla", palettes::crameri::LAJOLLA),
        ("seq-thermal", palettes::oceanic::THERMAL),
        ("seq-cividis", palettes::singles::CIVIDIS),
        ("seq-inferno", palettes::viridis::INFERNO),
        ("seq-viridis", palettes::viridis::VIRIDIS),
        ("seq-batlow_k", palettes::crameri::BATLOW_K),
        ("seq-batlow_w", palettes::crameri::BATLOW_W),
        // Diverging
        ("div-bam", palettes::crameri::BAM),
        ("div-vik", palettes::crameri::VIK),
        ("div-broc", palettes::crameri::BROC),
        ("div-cork", palettes::crameri::CORK),
        ("div-roma", palettes::crameri::ROMA),
        ("div-curl", palettes::oceanic::CURL),
        ("div-diff", palettes::oceanic::DIFF),
        ("div-tarn", palettes::oceanic::TARN),
        ("div-delta", palettes::oceanic::DELTA),
        ("div-berlin", palettes::crameri::BERLIN),
        ("div-lisbon", palettes::crameri::LISBON),
        ("div-tofino", palettes::crameri::TOFINO),
        ("div-vanimo", palettes::crameri::VANIMO),
        ("div-balance", palettes::oceanic::BALANCE),
        // Dual-sequential
        ("dual-topo", palettes::oceanic::TOPO),
        ("dual-fes", palettes::crameri::FES),
        ("dual-oleron", palettes::crameri::OLERON),
        ("dual-bukavu", palettes::crameri::BUKAVU),
        // Cyclic
        ("cyc-bam_o", palettes::crameri::BAM_O),
        ("cyc-vik_o", palettes::crameri::VIK_O),
        ("cyc-phase", palettes::oceanic::PHASE),
        ("cyc-broc_o", palettes::crameri::BROC_O),
        ("cyc-cork_o", palettes::crameri::CORK_O),
        ("cyc-roma_o", palettes::crameri::ROMA_O),
        // Diagnostic
        ("diag-byte_value", palettes::diagnostic::BYTE_VALUE),
    ];

    for (name, palette) in &builtin {
        let path = out_dir.join(format!("{name}.png"));
        save_gradient(&path, palette);
        println!("  {}", path.display());
    }

    // ── Perceptual colormaps ───────────────────────────────────────────

    let perceptual_sets: Vec<(&str, Vec<RGB8>)> = vec![
        // 1 color (auto-brackets with black/white)
        ("perceptual-1-red", vec![
            c(0xFF, 0x00, 0x00),
        ]),
        ("perceptual-1-teal", vec![
            c(0x00, 0x99, 0x88),
        ]),
        // 2 colors
        ("perceptual-2-blue-yellow", vec![
            c(0x00, 0x00, 0xFF),
            c(0xFF, 0xFF, 0x00),
        ]),
        ("perceptual-2-black-white", vec![
            c(0x00, 0x00, 0x00),
            c(0xFF, 0xFF, 0xFF),
        ]),
        // 3 colors (the "user's favourite" use case)
        ("perceptual-3-white-blue-black", vec![
            c(0xFF, 0xFF, 0xFF),
            c(0x3B, 0x82, 0xF6),
            c(0x00, 0x00, 0x00),
        ]),
        ("perceptual-3-black-red-white", vec![
            c(0x00, 0x00, 0x00),
            c(0xFF, 0x00, 0x00),
            c(0xFF, 0xFF, 0xFF),
        ]),
        ("perceptual-3-fire", vec![
            c(0x00, 0x00, 0x00),
            c(0xFF, 0x66, 0x00),
            c(0xFF, 0xFF, 0xCC),
        ]),
        // 4 colors
        ("perceptual-4-rainbow-ends", vec![
            c(0x00, 0x00, 0x00),
            c(0xFF, 0x00, 0x00),
            c(0x00, 0xFF, 0x00),
            c(0xFF, 0xFF, 0xFF),
        ]),
        // 5 colors
        ("perceptual-5-full-rainbow", vec![
            c(0xFF, 0x00, 0x00),
            c(0xFF, 0xFF, 0x00),
            c(0x00, 0xFF, 0x00),
            c(0x00, 0x00, 0xFF),
            c(0xFF, 0x00, 0xFF),
        ]),
        // 6 colors — scrambled to show sort difference
        ("perceptual-6-scrambled", vec![
            c(0xFF, 0xFF, 0x00),
            c(0x00, 0x00, 0x00),
            c(0xFF, 0x00, 0xFF),
            c(0xFF, 0xFF, 0xFF),
            c(0x00, 0x80, 0x00),
            c(0x00, 0x00, 0xFF),
        ]),
        // 8 colors — pastels
        ("perceptual-8-pastels", vec![
            c(0xFF, 0xAD, 0xAD),
            c(0xFF, 0xD6, 0xA5),
            c(0xFD, 0xFF, 0xB6),
            c(0xCA, 0xFF, 0xBF),
            c(0x9B, 0xF6, 0xFF),
            c(0xA0, 0xC4, 0xFF),
            c(0xBD, 0xB2, 0xFF),
            c(0xFF, 0xC6, 0xFF),
        ]),
    ];

    for (name, colors) in &perceptual_sets {
        // Unsorted
        let palette = perceptual::generate(colors);
        let path = out_dir.join(format!("{name}.png"));
        save_gradient(&path, &palette);
        println!("  {}", path.display());

        // Sorted
        if colors.len() > 2 {
            let sorted = perceptual::sort_colors(colors);
            let palette = perceptual::generate(&sorted);
            let path = out_dir.join(format!("{name}-sorted.png"));
            save_gradient(&path, &palette);
            println!("  {}", path.display());
        }
    }

    let total = builtin.len()
        + perceptual_sets.len()
        + perceptual_sets.iter().filter(|(_, c)| c.len() > 2).count();
    println!("\nGenerated {total} palette images in {}", out_dir.display());
}

fn c(r: u8, g: u8, b: u8) -> RGB8 {
    RGB8::new(r, g, b)
}

fn save_gradient(path: &Path, palette: &[u8]) {
    assert_eq!(palette.len(), 768);
    let mut img = ImageBuffer::new(256, 64);
    for x in 0..256u32 {
        let i = x as usize * 3;
        let pixel = Rgb([palette[i], palette[i + 1], palette[i + 2]]);
        for y in 0..64u32 {
            img.put_pixel(x, y, pixel);
        }
    }
    img.save(path).unwrap();
}
