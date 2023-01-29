use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use rand::{thread_rng, Rng};

use std::ffi::c_int;

use std::time::Instant;

// taken from MSVC
const RAND_MAX: c_int = 32767;

fn bench_caller(name: &str, sorter: fn(&mut [c_int]), c: &mut Criterion, data_size: i32) {
    c.bench_function(
        &format!("{name} n = {data_size}",),
        move |bencher: &mut Bencher| {
            bencher.iter_custom(|iterations| {
                let mut rng = thread_rng();
                let mut bench_data = (0..iterations)
                    .map(|_| {
                        let mut data = Vec::with_capacity(data_size as usize);
                        for _ in 0..data_size {
                            data.push((rng.gen_range(0..=RAND_MAX) % 1000) - 500)
                        }
                        data
                    })
                    .collect::<Vec<_>>();

                let origins = bench_data.clone();

                let start = Instant::now();
                for i in 0..iterations {
                    let data = &mut bench_data[i as usize];
                    sorter(data);
                }
                let time = start.elapsed();

                for (bench, mut origin) in bench_data.into_iter().zip(origins) {
                    origin.sort_unstable();
                    if bench != origin {
                        panic!("verifying failed: array is not sorted: {bench:#?}");
                    }
                }

                time
            })
        },
    );
}

fn bench(c: &mut Criterion) {
    let mut d = |f: fn(c_int, &mut Criterion)| {
        for mut i in 1..=10 {
            i *= 10;
            f(i, c);
        }
        for mut i in 2..=10 {
            i *= 100;
            f(i, c);
        }
        for mut i in 2..=10 {
            i *= 1000;
            f(i, c);
        }
        for mut i in 2..=10 {
            i *= 10000;
            f(i, c);
        }
    };

    d(|i, c| bench_caller("mysort", sorting_contest::mysort, c, i));
    d(|i, c| bench_caller("insertion sort", sorting_contest::insertion_sort, c, i));
    d(|i, c| bench_caller("quicksort", sorting_contest::quicksort, c, i));
    d(|i, c| bench_caller("heapsort", sorting_contest::heapsort, c, i));
    d(|i, c| bench_caller("counting sort", sorting_contest::counting_sort, c, i));
}

criterion_group!(benches, bench);
criterion_main!(benches);

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn counting_sort_test() {
        const COUNTING_SORT_ELEMENT_SIZE: usize = 128000;

        macro_rules! sort {
            ($array:expr) => {{
                let mut a = $array;
                sorting_contest::counting_sort(&mut a);
                a
            }};
        }

        let empty: [c_int; 0] = []; // type inference fails
        assert_eq!(sort!(empty), empty);

        assert_eq!(sort!([0]), [0]);
        assert_eq!(sort!([1, 2]), [1, 2]);
        assert_eq!(sort!([5, 8, 9, 3, 5]), [3, 5, 5, 8, 9]);
        assert_eq!(
            sort!([10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        );
        assert_eq!(
            sort!([5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5]),
            [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5]
        );

        let mut rng = thread_rng();

        // fuzz
        for _ in 0..3000 {
            let mut data = (0..rng.gen_range(0..3000)).collect::<Vec<c_int>>();
            for d in &mut data {
                let size: c_int = COUNTING_SORT_ELEMENT_SIZE as _;
                *d = rng.gen_range((-(size / 2))..=(size / 2));
            }

            let mut origin = data.clone();
            origin.sort_unstable();

            sorting_contest::counting_sort(&mut data);

            assert_eq!(data, origin);
        }
    }

    #[test]
    fn insertion_sort_test() {
        generic_test_sort(sorting_contest::insertion_sort);
    }
    #[test]
    fn heapsort_test() {
        generic_test_sort(sorting_contest::heapsort);
    }
    #[test]
    fn mysort_test() {
        generic_test_sort(sorting_contest::mysort);
    }

    fn generic_test_sort(sort_fn: fn(&mut [i32])) {
        macro_rules! sort {
            ($array:expr) => {{
                let mut a = $array;
                sort_fn(&mut a);
                a
            }};
        }

        let empty: [c_int; 0] = []; // type inference fails
        assert_eq!(sort!(empty), empty);

        assert_eq!(sort!([0]), [0]);
        assert_eq!(sort!([1, 2]), [1, 2]);
        assert_eq!(sort!([5, 8, 9, 3, 5]), [3, 5, 5, 8, 9]);
        assert_eq!(
            sort!([10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
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
}
