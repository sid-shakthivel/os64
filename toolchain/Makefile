AUTOMAKE = automake-1.11
AUTOCONF = autoconf-2.65
BINUTILS = binutils-2.38
GCC = gcc-11.2.0

TARGET = x86_64-elf
PREFIX = $(shell pwd)/cross

ROOT = $(shell pwd)

all: $(AUTOCONF) $(AUTOMAKE) $(BINUTILS) $(GCC)

$(AUTOCONF):
	mkdir -p $(PREFIX) && \
	curl -O -L http://mirror.hostiran.ir/gnu/gnu/autoconf/$(AUTOCONF).tar.gz && \
	tar -xzf $(AUTOCONF).tar.gz && \
	cd $(AUTOCONF) && \
	./configure && \
	make && \
	make install && \
	cd $(ROOT) 

$(AUTOMAKE):
	mkdir -p $(PREFIX) && \
	curl -O -L http://gnu.bardia.tech/gnu/automake/$(AUTOMAKE).tar.gz && \
	tar -xzf $(AUTOMAKE).tar.gz && \
	cd $(AUTOMAKE) && \
	./configure && \
	make && \
	sudo make install && \
	cd $(ROOT)

$(BINUTILS):
	mkdir -p $(PREFIX) && \
	curl -O -L https://ftp.wayne.edu/gnu/binutils/$(BINUTILS).tar.gz && \
	tar -xzf $(BINUTILS).tar.gz && \
	cd $(BINUTILS) && \
	./configure  --target=$(TARGET) --prefix=$(PREFIX) --with-sysroot --disable-multilib --disable-nls --disable-werror && \
	make && \
	make install && \
	cd $(ROOT)

$(GCC): $(BINUTILS)
	mkdir -p $(PREFIX) && \
	curl -O -L https://ftp.wayne.edu/gnu/gcc/gcc-11.2.0/$(GCC).tar.gz && \
	tar -xzf $(GCC).tar.gz && \
	cd $(GCC) && \
	./configure --target=$(TARGET) --prefix=$(PREFIX) --disable-nls --disable-multilib --enable-languages=c,c++ --without-headers && \
	make all-gcc && \
	make all-target-libgcc && \
	make install-gcc && \
	make install-target-libgcc && \
	cd $(ROOT)

clean:
	rm -f $(AUTOCONF).tar.gz
	rm -f $(AUTOMAKE).tar.gz
	rm -f $(BINUTILS).tar.gz
	rm -f $(GCC).tar.gz
	rm -rf $(PREFIX)
	# rm -rf $(AUTOCONF)
	# rm -rf $(AUTOMAKE)
	# rm -rf $(BINUTILS)
	# rm -rf $(GCC)

