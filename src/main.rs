use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use rand::Rng;
use std::ffi::c_int;
use std::time::Instant;

// taken from MSVC
const RAND_MAX: c_int = 32767;

fn mysort(data: &mut [c_int]) {
    extern "C" {
        fn mysort(data: *mut c_int, len: c_int);
    }
    unsafe {
        mysort(data.as_mut_ptr(), data.len() as i32);
    }
}

fn bench_caller(c: &mut Criterion, data_size: i32) {
    c.bench_function(
        &format!("mysort n = {data_size}",),
        move |bencher: &mut Bencher| {
            bencher.iter_custom(|iterations| {
                let mut rng = rand::thread_rng();
                let mut bench_data = (0..iterations)
                    .map(|_| {
                        let mut data = Vec::with_capacity(data_size as usize);
                        for _ in 0..data_size {
                            data.push(rng.gen_range(0..RAND_MAX) % 1000)
                        }
                        data
                    })
                    .collect::<Vec<_>>();

                let start = Instant::now();
                for i in 0..iterations {
                    let data = &mut bench_data[i as usize];
                    black_box(mysort(data));
                }
                start.elapsed()
            })
        },
    );
}

fn bench(c: &mut Criterion) {
    bench_caller(c, 100);
    bench_caller(c, 10_000);
    // skipped, too slow.
    // bench_caller(c, 100_000);
}

criterion_group!(benches, bench);
criterion_main!(benches);

#[test]
fn test_mysort() {
    use pretty_assertions::assert_eq;
    fn test<const N: usize>(mut s: [i32; N]) -> [i32; N] {
        mysort(&mut s);
        s
    }

    let empty: [i32; 0] = []; // type inference fails
    assert_eq!(test(empty), empty);

    assert_eq!(test([0]), [0]);
    assert_eq!(test([1, 2]), [1, 2]);
    assert_eq!(test([5, 8, 9, 3, 5]), [3, 5, 5, 8, 9]);
    assert_eq!(
        test([10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    );
}
