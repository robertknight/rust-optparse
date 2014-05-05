LIB_SRCS=*.rs

demo: *.rc $(LIB_SRCS)
	rustc demo.rc

doc: $(LIB_SRCS)
	rustdoc lib.rs

clean:
	rm -f demo
	rm -rf doc
