LIB_SRCS=*.rs

demo: demo.rs $(LIB_SRCS)
	rustc demo.rs

doc: $(LIB_SRCS)
	rustdoc lib.rs

clean:
	rm -f demo
	rm -rf doc
