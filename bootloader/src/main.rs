#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[macro_use]
mod print;

mod disk;
use disk::DiskReader;

mod gdt;
use gdt::GlobalDescriptorTable;

//const VERSION: &str = env!("CARGO_PKG_VERSION");
const KERNEL_LBA: u64 = 4096; //kernel location logical block address

const KERNEL_SIZE: u16 = 128; //kernel size in sectors

//TODO: Remember to uncomment this when kernel gets bigger
//const KERNEL_SIZE: u16 = 32768; //kernel size in sectors

const KERNEL_BUFFER: u16 = 0xbe00; //buffer location for copy
const KERNEL_TARGET: u32 = 0x0010_0000; //where to put kernel in memory

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}

//bootloader entry point
#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() -> ! {
    //unreal mode is needed because diskreader needs to copy from buffer to protected mode memory
    println!("[!] Switching to 16bit unreal mode...");
    unreal_mode();

    //load kernel
    print!("[!] Loading kernel");
    let mut disk = DiskReader::new(KERNEL_LBA, KERNEL_BUFFER);
    disk.read_sectors(KERNEL_SIZE, KERNEL_TARGET);
    println!("[!] Kernel loaded to memory.");

    //load dgt
    println!("[!] Loading Global Descriptor Table...");
    let gdt = GlobalDescriptorTable::new();
    gdt.load();

    //switch to protected mode
    println!("[!] Switching to 32bit protected mode and jumping to kernel...");
    protected_mode();

    //loop in case kernel returns
    loop {}
}

#[no_mangle]
pub extern "C" fn fail() -> ! {
    println!("[!] Read fail!");

    loop {}
}

//switch to 32bit protected mode and jump to kernel
fn protected_mode() {
    unsafe {
        //enable protected mode in cr0 register
        asm!("mov eax, cr0", "or al, 1", "mov cr0, eax");

        //push kernel address
        asm!(
            "push {0:e}",
            in(reg) KERNEL_TARGET,
        );

        //jump to protected mode
        asm!("ljmp $0x8, $2f", "2:", options(att_syntax));

        //protected mode start
        asm!(
            ".code32",

            //setup segment registers
            "mov {0:e}, 0x10",
            "mov ds, {0:e}",
            "mov es, {0:e}",
            "mov ss, {0:e}",

            //jump to kernel
            "pop {1:e}",
            "call {1:e}",

            out(reg) _,
            in(reg) KERNEL_TARGET,
        );
    }
}

//switch to 16bit unreal mode, this mode allows to use 32bit registers in 16bit mode
fn unreal_mode() {
    //backup segment registers
    let ds: u16;
    let ss: u16;
    unsafe {
        asm!("mov {0:x}, ds", out(reg) ds);
        asm!("mov {0:x}, ss", out(reg) ss);
    }

    //load gdt
    let gdt = GlobalDescriptorTable::new();
    gdt.load();

    unsafe {
        //backup cr0 register
        let mut cr0: u32;
        asm!("mov {0:e}, cr0", out(reg) cr0);

        //set cr0 protected bit
        let cr0_protected = cr0 | 1;
        asm!("mov cr0, {0:e}", in(reg) cr0_protected);

        //setup segment registers
        asm!("mov {0:x}, 0x10", "mov ds, {0:x}", "mov ss, {0:x}", out(reg) _);

        //restore cr0 register
        asm!("mov cr0, {0:e}", in(reg) cr0);

        //restore segment registers
        asm!("mov ds, {0:x}", in(reg) ds);
        asm!("mov ss, {0:x}", in(reg) ss);
    }
}
