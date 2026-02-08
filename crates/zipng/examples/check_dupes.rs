use zipng::palettes;

fn main() {
    let builtin: Vec<(&str, &[u8])> = vec![
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
        ("dual-topo", palettes::oceanic::TOPO),
        ("dual-fes", palettes::crameri::FES),
        ("dual-oleron", palettes::crameri::OLERON),
        ("dual-bukavu", palettes::crameri::BUKAVU),
        ("cyc-bam_o", palettes::crameri::BAM_O),
        ("cyc-vik_o", palettes::crameri::VIK_O),
        ("cyc-phase", palettes::oceanic::PHASE),
        ("cyc-broc_o", palettes::crameri::BROC_O),
        ("cyc-cork_o", palettes::crameri::CORK_O),
        ("cyc-roma_o", palettes::crameri::ROMA_O),
        ("diag-byte_value", palettes::diagnostic::BYTE_VALUE),
    ];

    let mut any_dupes = false;
    for (name, palette) in &builtin {
        let mut dupes = 0;
        for i in 1..256 {
            let a = &palette[(i-1)*3..(i-1)*3+3];
            let b = &palette[i*3..i*3+3];
            if a == b {
                dupes += 1;
            }
        }
        if dupes > 0 {
            any_dupes = true;
            println!("{name}: {dupes} adjacent duplicate(s)");
        }
    }
    if !any_dupes {
        println!("No adjacent duplicates in any built-in palette.");
    }
}
