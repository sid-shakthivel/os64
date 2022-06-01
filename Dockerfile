FROM ubuntu:18.04

# Install dependencies
RUN apt-get update -y                             

# Install Xorriso
RUN apt-get install xorriso -y

# Install Grub
RUN apt-get install grub-common -y
RUN apt-get install grub-pc-bin -y

# Install Binutils
RUN apt-get install binutils -y

WORKDIR /

# Create ISO

# CMD cd code && grub-mkrescue -o os64.iso isodir
