use super::*;

#[test]
fn rng_is_deterministic() {
    let mut a = Rng::seed(42);
    let mut b = Rng::seed(42);
    for _ in 0..1000 {
        assert_eq!(a.next(), b.next());
    }
    assert_ne!(Rng::seed(1).next(), Rng::seed(2).next());
}

// Known-answer vector: seed 42 must yield this exact sequence (cross-run repro).
const KAT_SEED_42: [u64; 8] = [
    0xBDD7_3226_2FEB_6E95,
    0xDB45_1835_B9F4_A0E1,
    0x18CC_6D35_9283_16D8,
    0x06B2_26D8_C070_CCCA,
    0xE79D_500A_13E1_FE95,
    0xA1E3_EA38_9E3D_EB2B,
    0x0D03_F88A_8CB9_A563,
    0xEF7D_3D6F_C0AF_8C0E,
];

#[test]
fn rng_matches_known_answer_vector() {
    let mut r = Rng::seed(42);
    for expected in KAT_SEED_42 {
        assert_eq!(r.next(), expected);
    }
}

#[test]
fn havoc_never_panics_and_clamps() {
    let dict = vec![b"SELECT".to_vec(), b"".to_vec()];
    for seed in [42u64, 7, 99999] {
        let mut rng = Rng::seed(seed);
        for input in [&b""[..], &b"a"[..], &b"abcd"[..], &[0u8; 200][..]] {
            for _ in 0..500 {
                assert!(havoc(input, &dict, &mut rng).len() <= super::constants::MAX_FILE);
            }
        }
    }
}

#[test]
fn splice_joins_or_declines() {
    let mut rng = Rng::seed(1);
    assert!(splice(b"a", b"bbbb", &mut rng).is_none());
    assert!(splice(b"AAAAAA", b"BBBBBB", &mut rng)
        .map(|v| !v.is_empty())
        .unwrap_or(true));
}

#[test]
fn energy_within_caps() {
    for fuzz_level in 0..40u32 {
        for n_fuzz in [1u32, 2, 7, 1000, u32::MAX] {
            let seed = Seed {
                buf: vec![],
                fuzz_level,
                n_fuzz,
                favored: false,
                barren: false,
            };
            assert!((1..=1600).contains(&energy(&seed)));
        }
    }
}
