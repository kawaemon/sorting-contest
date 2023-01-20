# vim: ft=make
bench:
	rm -rf target/criterion
	cargo run --bin bench -- --quiet --bench --measurement-time 10 > performance
	cat performance

fmt:
	cargo fmt
	clang-format -i mysort.c
