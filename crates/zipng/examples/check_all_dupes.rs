use zipng::palettes;
use std::collections::HashSet;

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
    ];

    let mut any_nonunique = false;
    for (name, palette) in &builtin {
        let mut unique_colors = HashSet::new();
        let mut duplicate_count = 0;
        
        for i in 0..256 {
            let color = &palette[i*3..i*3+3];
            if !unique_colors.insert(color) {
                duplicate_count += 1;
            }
        }
        
        if duplicate_count > 0 {
            any_nonunique = true;
            println!("{name}: {duplicate_count} total duplicate color(s) (non-unique indices)");
        }
    }
    
    if !any_nonunique {
        println!("All built-in palettes have 256 unique colors each!");
    }
}
