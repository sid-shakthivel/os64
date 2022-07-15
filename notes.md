### List of useful resources

Bochs magic breakpoint is xchg bx, bx
Dart: git clone https://kernel.googlesource.com/pub/scm/utils/dash/dash 

https://fejlesztek.hu/create-a-fat-file-system-image-on-linux/
https://stackoverflow.com/questions/22385189/add-files-to-vfat-image-without-mounting/29798605#29798605

Tasks:
Unix everything is a file
Make larger file of multiple clusters so we can loop through cluster chains (0x20)
Check if file or directory - if file then we can read the cluster else we must loop through it with the StandardDirectoryEntry
Create a graph of nodes of files and then map and have methods to read

Tomorrow:
Make methods to write files

Each cluster is 2048B