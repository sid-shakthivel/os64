ASSEMBLER = nasm
ASSEMBLER_FLAGS = -f bin

program: program2.asm
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) program2.asm -o program2
	rm -f ../../isodir/modules/program2
	mv program2 ../../isodir/modules