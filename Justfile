# vim: ft=make
bench:
	rm -rf target/criterion
	cargo run -- --quiet --bench --measurement-time 10 > performance
	cat performance
