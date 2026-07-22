.PHONY: clean run

run:
	cargo run --release

clean:
	rm -f *.ppm
