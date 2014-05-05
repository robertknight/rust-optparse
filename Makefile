demo: *.rc *.rs
	rustc demo.rc

doc: *.rs
	rustdoc lib.rs
