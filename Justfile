# vim: ft=make
bench:
	rm -rf target/criterion
	cargo run --release --bin bench -- --quiet --bench --measurement-time 10 --nocapture | tee performance

fmt:
	cargo fmt
	clang-format -i mysort.c
