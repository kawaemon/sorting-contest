use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use rand::{thread_rng, Rng};
use sorting_contest::mysort;
use std::ffi::c_int;

use std::time::Instant;

// taken from MSVC
const RAND_MAX: c_int = 32767;

fn bench_caller(c: &mut Criterion, data_size: i32) {
    c.bench_function(
        &format!("mysort n = {data_size}",),
        move |bencher: &mut Bencher| {
            bencher.iter_custom(|iterations| {
                let mut rng = thread_rng();
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
    bench_caller(c, 100_000);
}

criterion_group!(benches, bench);
criterion_main!(benches);

#[cfg(test)]
#[test]
fn insertion_sort_test() {
    test_sort(sorting_contest::insertion_sort);
}
#[cfg(test)]
#[test]
fn heapsort_test() {
    test_sort(sorting_contest::heapsort);
}
#[cfg(test)]
#[test]
fn mysort_test() {
    test_sort(mysort)
}

#[cfg(test)]
fn test_sort(sort_fn: fn(&mut [i32])) {
    use pretty_assertions::assert_eq;

    macro_rules! test {
        ($array:expr) => {{
            let mut a = $array;
            sort_fn(&mut a);
            a
        }};
    }

    let empty: [c_int; 0] = []; // type inference fails
    assert_eq!(test!(empty), empty);

    assert_eq!(test!([0]), [0]);
    assert_eq!(test!([1, 2]), [1, 2]);
    assert_eq!(test!([5, 8, 9, 3, 5]), [3, 5, 5, 8, 9]);
    assert_eq!(
        test!([10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    );

    let mut rng = thread_rng();

    // fuzz
    for _ in 0..3000 {
        let mut data = (0..3000).collect::<Vec<c_int>>();
        rng.fill(data.as_mut_slice());

        let mut origin = data.clone();
        origin.sort_unstable();

        sort_fn(&mut data);

        assert_eq!(data, origin);
    }
}
