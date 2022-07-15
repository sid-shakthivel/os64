### List of useful resources

Bochs magic breakpoint is xchg bx, bx
Dart: git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash 

https://fejlesztek.hu/create-a-fat-file-system-image-on-linux/
https://stackoverflow.com/questions/22385189/add-files-to-vfat-image-without-mounting/29798605#29798605

Tasks:
Make larger file of multiple clusters so we can loop through cluster chains (0x20)
Create a graph of nodes of files and then map and have methods to read with VFS

Tomorrow:
Make methods to write files

Each cluster is 2048B
2 nodes - file node which stores cluster number
        - folder node which stores folders