use ::save::{
    testing::assert_at,
    zigzag::{ZigZag, ZugZug},
};
use inline::snapshot;

#[test]
fn zigzag_round_trip() {
    for uint in u8::MIN..=u8::MAX {
        assert_eq!(uint, uint.zigzag().zigzag());
    }
    for int in i8::MIN..=i8::MAX {
        assert_eq!(int, int.zigzag().zigzag());
    }
    for uint in u16::MIN..=u16::MAX {
        assert_eq!(uint, uint.zigzag().zigzag());
    }
    for int in i16::MIN..=i16::MAX {
        assert_eq!(int, int.zigzag().zigzag());
    }
}

#[test]
fn zigzag_known_values() {
    snapshot("0".to_string()).value = format!("{:?}", 0_u8.zigzag());
    snapshot("0".to_string()).value = format!("{:?}", 0_u16.zigzag());
    snapshot("0".to_string()).value = format!("{:?}", 0_u32.zigzag());
    snapshot("0".to_string()).value = format!("{:?}", 0_u64.zigzag());
    snapshot("0".to_string()).value = format!("{:?}", 0_u128.zigzag());
    snapshot("0".to_string()).value = format!("{:?}", 0_usize.zigzag());
    snapshot("1".to_string()).value = format!("{:?}", 1_u8.zigzag());
    snapshot("-1".to_string()).value = format!("{:?}", 2_u8.zigzag());
    snapshot("2".to_string()).value = format!("{:?}", 3_u8.zigzag());
    snapshot("-2".to_string()).value = format!("{:?}", 4_u8.zigzag());
    snapshot("3".to_string()).value = format!("{:?}", 5_u8.zigzag());
    snapshot("63".to_string()).value = format!("{:?}", 125_u8.zigzag());
    snapshot("-63".to_string()).value = format!("{:?}", 126_u8.zigzag());
    snapshot("64".to_string()).value = format!("{:?}", 127_u8.zigzag());
    snapshot("-64".to_string()).value = format!("{:?}", 128_u8.zigzag());
    snapshot("65".to_string()).value = format!("{:?}", 129_u8.zigzag());
    snapshot("-65".to_string()).value = format!("{:?}", 130_u8.zigzag());
    snapshot("-125".to_string()).value = format!("{:?}", 250_u8.zigzag());
    snapshot("126".to_string()).value = format!("{:?}", 251_u8.zigzag());
    snapshot("-126".to_string()).value = format!("{:?}", 252_u8.zigzag());
    snapshot("127".to_string()).value = format!("{:?}", 253_u8.zigzag());
    snapshot("-127".to_string()).value = format!("{:?}", 254_u8.zigzag());
    snapshot("-128".to_string()).value = format!("{:?}", 255_u8.zigzag());
    snapshot("128".to_string()).value = format!("{:?}", 255_u16.zigzag());
    snapshot("128".to_string()).value = format!("{:?}", 255_u32.zigzag());
    snapshot("128".to_string()).value = format!("{:?}", 255_u64.zigzag());
    snapshot("128".to_string()).value = format!("{:?}", 255_u128.zigzag());
    snapshot("128".to_string()).value = format!("{:?}", 255_usize.zigzag());
    assert_eq!(u8::MAX.zigzag(), i8::MIN);
    assert_eq!(u16::MAX.zigzag(), i16::MIN);
    assert_eq!(u32::MAX.zigzag(), i32::MIN);
    assert_eq!(u64::MAX.zigzag(), i64::MIN);
    assert_eq!(u128::MAX.zigzag(), i128::MIN);
    assert_eq!(usize::MAX.zigzag(), isize::MIN);
    assert_eq!((u8::MAX - 2).zigzag(), i8::MAX);
    assert_eq!((u16::MAX - 2).zigzag(), i16::MAX);
    assert_eq!((u32::MAX - 2).zigzag(), i32::MAX);
    assert_eq!((u64::MAX - 2).zigzag(), i64::MAX);
    assert_eq!((u128::MAX - 2).zigzag(), i128::MAX);
    assert_eq!((usize::MAX - 2).zigzag(), isize::MAX);
}

#[test]
fn zigzag_snapshot() {
    let mut actual = String::new();
    for i in u8::MIN..=u8::MAX {
        let x = i.zigzag();
        let xp = (i as u16).zigzag();
        actual += &format!("{i:>4}_u8.zigzag() == {x:>4}_i8");
        if xp != (x as _) {
            actual += &format!(", but {i:>4}_u16.zigzag() == {xp:>4}_i16");
        }
        actual += "\n";
    }
    actual += "\n";
    for i in i8::MIN..=i8::MAX {
        let x = i.zigzag();
        let xp = (i as i16).zigzag();
        actual += &format!("{i:>4}_i8.zigzag() == {x:>4}_u8");
        if xp != (x as _) {
            actual += &format!(", but {i:>4}_i16.zigzag() == {xp:>4}_u16");
        }
        actual += "\n";
    }
    assert_at("zigzag.txt", &actual);
}

#[test]
fn zugzug_round_trip() {
    for uint in u8::MIN..=u8::MAX {
        assert_eq!(uint, uint.zugzug().zugzug());
    }
    for int in i8::MIN..=i8::MAX {
        assert_eq!(int, int.zigzag().zugzug().zugzug().zigzag());
    }
    for uint in u16::MIN..=u16::MAX {
        assert_eq!(uint, uint.zugzug().zugzug());
    }
    for int in i16::MIN..=i16::MAX {
        assert_eq!(int, int.zigzag().zugzug().zugzug().zigzag());
    }
}

#[test]
fn zugzug_known_values() {
    snapshot("(0, 0)".to_string()).value = format!("{:?}", 0_u8.zugzug());
    snapshot("(0, 1)".to_string()).value = format!("{:?}", 1_u8.zugzug());
    snapshot("(1, 1)".to_string()).value = format!("{:?}", 2_u8.zugzug());
    snapshot("(-1, 0)".to_string()).value = format!("{:?}", 3_u8.zugzug());
    snapshot("(-1, 1)".to_string()).value = format!("{:?}", 4_u8.zugzug());
    snapshot("(-1, -1)".to_string()).value = format!("{:?}", 5_u8.zugzug());
    snapshot("(0, 2)".to_string()).value = format!("{:?}", 6_u8.zugzug());
    snapshot("(1, 2)".to_string()).value = format!("{:?}", 7_u8.zugzug());
    snapshot("(-1, 2)".to_string()).value = format!("{:?}", 8_u8.zugzug());
    snapshot("(2, 2)".to_string()).value = format!("{:?}", 9_u8.zugzug());
    snapshot("(-2, 0)".to_string()).value = format!("{:?}", 10_u8.zugzug());
    snapshot("(-2, 1)".to_string()).value = format!("{:?}", 11_u8.zugzug());
    snapshot("(-11, 1)".to_string()).value = format!("{:?}", 254_u8.zugzug());
    snapshot("(-11, -1)".to_string()).value = format!("{:?}", 255_u8.zugzug());
    assert_eq!(u8::MAX.zugzug(), (-11_i8, -1_i8));
    assert_eq!(u16::MAX.zugzug(), (-97_i16, 181_i16));
    assert_eq!(u32::MAX.zugzug(), (-18537_i32, 46341_i32));
    assert_eq!(u64::MAX.zugzug(), (1373026058_i64, 3037000500_i64));
    assert_eq!(usize::MAX.zugzug(), (1373026058_isize, 3037000500_isize));
}

#[test]
fn zugzug_snapshot() {
    let mut actual = String::new();
    for i in 0..1024u16 {
        let (x, y) = i.zugzug();
        actual += &format!("{i:>4}.zugzug() == ({x:>3}, {y:>3})\n");
    }
    actual += "\n";
    for x in -16..16i16 {
        for y in x..32i16 {
            let i = (x, y).zugzug();
            actual += &format!("({x:>3}, {y:>3}).zugzug() == {i:>4}\n");
        }
    }
    assert_at("zugzug.txt", &actual);
}
