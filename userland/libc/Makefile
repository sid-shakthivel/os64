NEWLIB = newlib-4.1.0
TARGET = x86_64-sidos
SYSROOT = $(shell pwd)/build

all:
	mkdir -p newlib_build && \
	cd newlib_build && \
	../$(NEWLIB)/configure --prefix=/ --target=$(TARGET) && \
	make all && \
	make DESTDIR=$(SYSROOT) install

clean:
	rm -rf newlib_build
	rm -rf build
