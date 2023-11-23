FROM ubuntu:22.04
ARG DEBIAN_FRONTEND=noninteractive

# Make working directory
RUN mkdir riscv64-linux

# Build Linux kernel
WORKDIR /riscv64-linux/linux

RUN apt-get update && apt-get install -y libelf-dev libudev-dev libpci-dev libiberty-dev dkms openssl git flex bison libssl-dev libncurses-dev bc gcc-9-riscv64-linux-gnu gcc-riscv64-linux-gnu
RUN update-alternatives --install /usr/bin/riscv64-linux-gnu-gcc riscv64-linux-gnu-gcc /usr/bin/riscv64-linux-gnu-gcc-9 9

RUN git clone https://github.com/torvalds/linux.git --depth=1 --branch v5.4
RUN apt-get install -y curl
COPY linux.config /riscv64-linux/linux/linux/.config
RUN cd linux && \
    git config --global user.email "wessel@github.com" && git config --global user.name "wessel" && \
    curl -O https://github.com/torvalds/linux/commit/e33a814e772cdc36436c8c188d8c42d019fda639.patch && \
    git am e33a814e772cdc36436c8c188d8c42d019fda639.patch && \
    make ARCH=riscv CROSS_COMPILE=riscv64-linux-gnu- -j $(nproc)

# Build OpenSBI
WORKDIR /riscv64-linux
RUN apt-get install -y gcc-riscv64-unknown-elf
RUN git clone https://github.com/riscv/opensbi.git --depth=1 --branch v0.8
RUN cd opensbi && \
    make CROSS_COMPILE=riscv64-unknown-elf- PLATFORM=generic \
        FW_PAYLOAD_PATH=../linux/linux/arch/riscv/boot/Image

# Build Busybox
WORKDIR /riscv64-linux
RUN git clone https://github.com/mirror/busybox.git --depth=1 --branch=1_32_0
COPY busybox.config /riscv64-linux/busybox/.config
RUN cd busybox && make ARCH=riscv CROSS_COMPILE=riscv64-linux-gnu- install

# Make root file system image
WORKDIR /riscv64-linux/rootfs
COPY run.sh /riscv64-linux/run.sh

CMD /riscv64-linux/run.sh
