use zipng::polyglot::fonts::ALL_FONTS;
fn main() {
    let sugi = &***ALL_FONTS.iter().find(|f| f.name == "Sugimori Sans").unwrap();

    // Check all chars in the font
    let mut chars: Vec<char> = sugi.chars().collect();
    chars.sort();
    eprintln!("Total chars in font: {}", chars.len());
    for c in &chars {
        if *c as u32 > 0x2000 {
            eprintln!("  U+{:04X} {:?}", *c as u32, c);
        }
    }

    // Specifically check ♂ and ♀
    eprintln!("\n♀ U+2640 in chars: {}", chars.contains(&'♀'));
    eprintln!("♂ U+2642 in chars: {}", chars.contains(&'♂'));

    for c in ['♀', '♂'] {
        if let Some(l) = sugi.get_glyph(c) {
            eprintln!("\nget_glyph({c} U+{:04X}):", c as u32);
            eprintln!("  skip_kerning: {}", l.skip_kerning);
            for row in l.glyph {
                let s: String = row.iter().map(|&b| if b { '#' } else { '.' }).collect();
                eprintln!("  {s}");
            }
        }
    }
}
