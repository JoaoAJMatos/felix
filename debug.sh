#!/bin/bash

# Builds os and debugs it in Bochs
echo "Cleaning build directory..."
rm -rf build
echo "Building Felix..."
cargo build

echo "Making boot image..."
mkdir build

#this is needed to shrink the bootloader to 512 bytes
objcopy -I elf32-i386 -O binary target/x86_16-felix/debug/felix-boot build/boot.bin
objcopy -I elf32-i386 -O binary target/x86_16-felix/debug/felix-bootloader build/bootloader.bin

#create the disk image
#306 cylinders, 4 heads, 17 sectors per track => 20808 sectors in total
#dd if=/dev/zero of=build/disk.img bs=512 count=20808

#create floppy image
dd if=/dev/zero of=build/disk.img bs=512 count=2880
mkfs.fat -F 12 -n "FELIX" build/disk.img

#put the boot sector in first 512 bytes of disk...
dd if=build/boot.bin of=build/disk.img conv=notrunc

#put bootloader in the last 64 sectors of disk (2880 - 64)
dd if=build/bootloader.bin of=build/disk.img bs=512 seek=2816 conv=notrunc
#mcopy -i build/disk.img build/kernel.bin "::kernel.bin"
#dd if=build/kernel.bin of=build/disk.img bs=512 seek=32 conv=notrunc

echo "Debugging Felix with Bochs..."
bochs -q -f bochs.conf
#qemu-system-i386 -drive id=disk,file=build/disk.img,if=none,format=raw -device ahci,id=ahci -device ide-hd,drive=disk,bus=ahci.0