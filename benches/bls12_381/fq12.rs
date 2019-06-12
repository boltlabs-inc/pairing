use rand::{SeedableRng};
use rand_xorshift::{XorShiftRng};
use ff::{Rand, Field};
use pairing::bls12_381::*;

#[bench]
fn bench_fq12_add_assign(b: &mut ::test::Bencher) {
    const SAMPLES: usize = 1000;

    let mut rng = XorShiftRng::seed_from_u64(0x5dbe62598d313d76);

    let v: Vec<(Fq12, Fq12)> = (0..SAMPLES)
        .map(|_| (Fq12::rand(&mut rng), Fq12::rand(&mut rng)))
        .collect();

    let mut count = 0;
    b.iter(|| {
        let mut tmp = v[count].0;
        tmp.add_assign(&v[count].1);
        count = (count + 1) % SAMPLES;
        tmp
    });
}

#[bench]
fn bench_fq12_sub_assign(b: &mut ::test::Bencher) {
    const SAMPLES: usize = 1000;

    let mut rng = XorShiftRng::seed_from_u64(0x5dbe62598d313d76);

    let v: Vec<(Fq12, Fq12)> = (0..SAMPLES)
        .map(|_| (Fq12::rand(&mut rng), Fq12::rand(&mut rng)))
        .collect();

    let mut count = 0;
    b.iter(|| {
        let mut tmp = v[count].0;
        tmp.sub_assign(&v[count].1);
        count = (count + 1) % SAMPLES;
        tmp
    });
}

#[bench]
fn bench_fq12_mul_assign(b: &mut ::test::Bencher) {
    const SAMPLES: usize = 1000;

    let mut rng = XorShiftRng::seed_from_u64(0x5dbe62598d313d76);

    let v: Vec<(Fq12, Fq12)> = (0..SAMPLES)
        .map(|_| (Fq12::rand(&mut rng), Fq12::rand(&mut rng)))
        .collect();

    let mut count = 0;
    b.iter(|| {
        let mut tmp = v[count].0;
        tmp.mul_assign(&v[count].1);
        count = (count + 1) % SAMPLES;
        tmp
    });
}

#[bench]
fn bench_fq12_squaring(b: &mut ::test::Bencher) {
    const SAMPLES: usize = 1000;

    let mut rng = XorShiftRng::seed_from_u64(0x5dbe62598d313d76);

    let v: Vec<Fq12> = (0..SAMPLES).map(|_| Fq12::rand(&mut rng)).collect();

    let mut count = 0;
    b.iter(|| {
        let mut tmp = v[count];
        tmp.square();
        count = (count + 1) % SAMPLES;
        tmp
    });
}

#[bench]
fn bench_fq12_inverse(b: &mut ::test::Bencher) {
    const SAMPLES: usize = 1000;

    let mut rng = XorShiftRng::seed_from_u64(0x5dbe62598d313d76);

    let v: Vec<Fq12> = (0..SAMPLES).map(|_| Fq12::rand(&mut rng)).collect();

    let mut count = 0;
    b.iter(|| {
        let tmp = v[count].inverse();
        count = (count + 1) % SAMPLES;
        tmp
    });
}
