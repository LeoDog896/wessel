#!/bin/bash

# build the image
sudo dd if=/dev/zero of=rootfs.img bs=1M count=50
sudo mkfs.ext2 -L riscv-rootfs rootfs.img
sudo mkdir /mnt/rootfs
sudo mount rootfs.img /mnt/rootfs
sudo cp -ar ../busybox/_install/* /mnt/rootfs
sudo mkdir /mnt/rootfs/{dev,home,mnt,proc,sys,tmp,var}
sudo chown -R -h root:root /mnt/rootfs
sudo df /mnt/rootfs
sudo mount | grep rootfs
sudo umount /mnt/rootfs
sudo rmdir /mnt/rootfs

# send the artifacts
cp /riscv64-linux/opensbi/build/platform/generic/firmware/fw_payload.elf /artifacts
cp /riscv64-linux/rootfs/rootfs.img /artifacts

ls -l /artifacts
echo "Done!"
