FROM ubuntu:18.04

# Install dependencies
RUN apt-get update -y                             

# Install Curl
RUN apt-get install curl -y

# Install Xorriso
RUN apt-get install xorriso -y

# Install Grub
RUN apt-get install grub-common -y
RUN apt-get install grub-pc-bin -y

# Install Binutils
RUN apt-get install binutils -y

# Install Nasm
RUN apt-get install nasm -y

# Install Make
RUN apt-get install make -y

# Install fdisk
RUN apt-get install fdisk -y

# Install mkfs.vfat
RUN apt-get install dosfstools -y

# Install gcc
RUN apt install build-essential -y

# Install ldd
RUN apt-get install libc-bin -y

WORKDIR /

# Compile and create ISO
# CMD cd code/kernel && make
