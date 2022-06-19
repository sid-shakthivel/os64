ASSEMBLER = nasm
ASSEMBLER_FLAGS = -f bin

program: program.asm
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) program.asm -o program
	rm -f ../../isodir/modules/program
	mv program ../../isodir/modules
