FROM ubuntu:18.04
ARG DEBIAN_FRONTEND=noninteractive

ENV RISCV=/opt/riscv
ENV PATH=$RISCV/bin:$PATH
WORKDIR $RISCV

RUN apt update
RUN apt install -y autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev gawk build-essential bison flex texinfo gperf libtool patchutils bc zlib1g-dev libexpat-dev wget sudo

# Compile git with openssh

RUN wget https://raw.githubusercontent.com/paul-nelson-baker/git-openssl-shellscript/master/compile-git-with-openssl.sh
RUN chmod +x compile-git-with-openssl.sh
RUN DEBIAN_FRONTEND=noninteractive bash compile-git-with-openssl.sh -skiptests

# TODO: do something about this??
RUN git config --global http.postBuffer 1048576000
RUN git config --global https.postBuffer 1048576000
RUN git config --global pack.window 1
RUN git config --global http.version HTTP/1.1

RUN git clone --recursive https://github.com/riscv/riscv-gnu-toolchain

RUN cd riscv-gnu-toolchain && ./configure --prefix=/opt/riscv && make

# Set the working directory
WORKDIR /riscv64-linux

# Install required packages
RUN apt-get update && apt-get install -y \
    dh-autoreconf \
    libcurl4-openssl-dev \
    libncurses5-dev \
    libssl-dev \
    mtools \
    dosfstools \
    e2fsprogs \
    libssl-dev \
    tcl-dev \
    gettext \
    asciidoc \
    docbook2x \
    install-info \
    libexpat1-dev \
    libz-dev

# Make working directory
RUN mkdir riscv64-linux

# Build Linux kernel
WORKDIR /riscv64-linux/linux

RUN apt-get install -y libelf-dev libudev-dev libpci-dev libiberty-dev dkms openssl

RUN git clone https://github.com/torvalds/linux.git --depth=1 --branch v5.4 && \
    cd linux && \
    make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- defconfig && \
    make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- menuconfig && \
    make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- -j $(nproc)

# Build OpenSBI
WORKDIR /riscv64-linux
RUN git clone https://github.com/riscv/opensbi.git && \
    cd opensbi && \
    git checkout v0.8 && \
    make CROSS_COMPILE=riscv64-unknown-elf- PLATFORM=generic \
        FW_PAYLOAD_PATH=../linux/arch/riscv/boot/Image

# Build Busybox
WORKDIR /riscv64-linux
RUN git clone https://github.com/mirror/busybox.git && \
    cd busybox && \
    git checkout 1_32_0 && \
    make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- defconfig && \
    make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- menuconfig && \
    make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- install

# Make root file system image
WORKDIR /riscv64-linux/rootfs
RUN dd if=/dev/zero of=rootfs.img bs=1M count=50 && \
    mkfs.ext2 -L riscv-rootfs rootfs.img && \
    mkdir /mnt/rootfs && \
    mount rootfs.img /mnt/rootfs && \
    cp -ar ../busybox/_install/* /mnt/rootfs && \
    mkdir /mnt/rootfs/{dev,home,mnt,proc,sys,tmp,var} && \
    chown -R -h root:root /mnt/rootfs && \
    df /mnt/rootfs && \
    mount | grep rootfs && \
    umount /mnt/rootfs && \
    rmdir /mnt/rootfs

# Copy the files
WORKDIR /riscv64-linux
