# vim: ft=make
bench:
	rm -rf target/criterion
	cargo run -- --quiet --bench --measurement-time 10 > performance
	cat performance

fmt:
	cargo fmt
	clang-format -style="{BasedOnStyle: google, IndentWidth: 4}" -i mysort.c
