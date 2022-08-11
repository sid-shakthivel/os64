AUTOMAKE = automake-1.11
AUTOCONF = autoconf-2.65
BINUTILS = binutils-2.11.2

TARGET = x86_64-elf
PREFIX = $(shell pwd)/cross

ROOT = $(shell pwd)~

$(AUTOCONF):
	curl -O -L http://mirror.hostiran.ir/gnu/gnu/autoconf/autoconf-2.65.tar.gz
	tar -xzf $(AUTOCONF).tar.gz
	cd autoconf-*
	./configure
	make
	make install
	cd $(ROOT)

$(AUTOMAKE):
	curl -O -L http://gnu.bardia.tech/gnu/automake/automake-1.11.tar.gz
	tar -xzf $(AUTOMAKE).tar.gz
	cd automake*
	./configure
	make
	make install
	cd $(ROOT)

$(BINUTILS):
	curl -O -L https://ftp.wayne.edu/gnu/binutils/binutils-2.11.2.tar.gz
	ar -xzf $(BINUTILS).tar.gz
	cd binutils-*
	./configure  --target=$(TARGET) --prefix=$(PREFIX) --with-sysroot --disable-nls --disable-werror
	make
	make install
	cd $(ROOT)

clean:
	rm -f $(AUTOCONF).tar.gz
	rm -f $(AUTOMAKE).tar.gz
	rm -f $(BINUTILS).tar.gz

