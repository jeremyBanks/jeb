use z855::z855::{encode, decode};

#[test]
fn test_1153_bytes_original() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let seed = 81u64;
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let h = hasher.finish();
    let len = 1000 + (h % 1000) as usize;
    let mut data = Vec::with_capacity(len);
    let mut state = h;
    for _ in 0..len {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        data.push((state >> 33) as u8);
    }
    let data = &data[..1153];
    let encoded = encode(data);
    match decode(&encoded) {
        Ok(d) => { assert_eq!(d, data); eprintln!("OK"); }
        Err(e) => { eprintln!("Error: {:?}", e); panic!("decode failed"); }
    }
}
