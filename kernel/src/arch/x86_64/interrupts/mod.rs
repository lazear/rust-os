use crate::prelude::*;

pub fn enable() {
    unsafe {
        asm!("sti" :::: "volatile");
    }
}

pub fn disable() {
    unsafe {
        asm!("cli" :::: "volatile");
    }
}

/// Run a closure with interrupts disabled
pub fn critical_section<T, F: FnOnce() -> T>(f: F) -> T {
    disable();
    let r = f();
    enable();
    r
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Preserved {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Scratch {
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct InterruptErrorStack {
    pub fs: usize,
    pub scratch: Scratch,
    pub preserved: Preserved,

    pub error_code: usize,

    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,
    pub rsp: usize,
    pub ss: usize,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct InterruptStack {
    pub fs: usize,
    pub scratch: Scratch,
    pub preserved: Preserved,

    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,
    pub rsp: usize,
    pub ss: usize,
}

impl core::fmt::Debug for Preserved {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "rbx: {:#016X} rbp: {:#016X}\nr12: {:#016X} r13: {:#016X}\nr14: {:#016X} r15: {:#016X}\n",
        self.rbx, self.rbp, self.r12, self.r13, self.r14, self.r15)
    }
}

impl core::fmt::Debug for InterruptErrorStack {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            fmt,
            "error {:0X} occurred at {:0X}:0x{:#016X} with flags {:#016X}!\n{:?}\nrsp {:0X}:{:#016X}",
            self.error_code, self.cs, self.rip, self.rflags, self.preserved, self.ss, self.rsp,
        )
    }
}

impl core::fmt::Debug for InterruptStack {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            fmt,
            "error occurred at {:0X}:0x{:#016X} with flags {:#016X}!\n{:?}",
            self.cs, self.rip, self.rflags, self.preserved
        )
    }
}

macro_rules! push_preserved {
    () => {asm!(
        "push rbx
        push rbp
        push r12
        push r13
        push r14
        push r15"
        :::: "intel", "volatile"
    )};
}

macro_rules! pop_preserved {
    () => {asm!(
        "pop r15
        pop r14 
        pop r13 
        pop r12 
        pop rbp 
        pop rbx"
        :::: "intel", "volatile"
    )};
}

macro_rules! push_scratch {
    () => {asm!(
        "push rax 
        push rcx
        push rdx
        push rdi
        push rsi
        push r8
        push r9
        push r10
        push r11"
        :::: "intel", "volatile"
    )};
}

macro_rules! pop_scratch {
    () => {asm!(
        "pop r11
        pop r10
        pop r9
        pop r8
        pop rsi
        pop rdi 
        pop rdx 
        pop rcx
        pop rax"
        :::: "intel", "volatile"
    )};
}

macro_rules! push_fs {
    () => {asm!(
        "push fs
        mov rax, 0x20
        mov fs, rax"
        :::: "intel", "volatile"
    )};
}

macro_rules! pop_fs {
    () => {asm!("pop fs" :::: "intel", "volatile")};
}

macro_rules! iretq {
    () => {asm!("iretq" :::: "intel", "volatile")};
}

#[macro_export]
macro_rules! interrupt {
    ($name:ident, $stack:ident) => {
        interrupt!($name, $stack, {
            println!("CPU fault: {}\n{:?}", stringify!($name), $stack);
        //    asm!("hlt" :::: "intel", "volatile");
        });
    };
    ($name:ident, $func:block) => {
        #[naked]
        pub unsafe extern "C" fn $name() {
            #[inline(never)]
            unsafe fn inner() {
                $func
            }

            push_preserved!();
            push_scratch!();
            push_fs!();

            inner();

            pop_fs!();
            pop_scratch!();
            pop_preserved!();
            iretq!();
        }
    };
    ($name:ident, $stack:ident, $func:block) => {
        #[naked]
        pub unsafe extern "C" fn $name() {
            #[inline(never)]
            unsafe fn inner($stack: &mut $crate::arch::interrupts::InterruptStack) {
                $func
            }

            push_preserved!();
            push_scratch!();
            push_fs!();

            let rsp: usize;
            asm!("" : "={rsp}"(rsp) ::: "intel", "volatile");

            inner(&mut *(rsp as *mut $crate::arch::interrupts::InterruptStack));

            pop_fs!();
            pop_scratch!();
            pop_preserved!();
            iretq!();
        }
    };
}

#[macro_export]
macro_rules! interrupt_error {
    ($name:ident, $stack:ident) => {
        interrupt_error!($name, $stack, {
            println!("CPU fault: {}\n{:?}", stringify!($name), $stack);
            // asm!("hlt" :::: "intel", "volatile");
        });
    };
    ($name:ident, $func:block) => {
        #[naked]
        pub unsafe extern "C" fn $name() {
            #[inline(never)]
            unsafe fn inner() {
                $func
            }

            push_preserved!();
            push_scratch!();
            push_fs!();

            inner();

            pop_fs!();
            pop_scratch!();
            pop_preserved!();
            // pop off error code
            asm!("add rsp, 8" :::: "intel", "volatile");
            iretq!();
        }
    };
    ($name:ident, $stack:ident, $func:block) => {
        #[naked]
        pub unsafe extern "C" fn $name() {
            #[inline(never)]
            unsafe fn inner($stack: &mut $crate::arch::interrupts::InterruptErrorStack) {
                $func
            }

            push_preserved!();
            push_scratch!();
            push_fs!();

            let rsp: usize;
            asm!("" : "={rsp}"(rsp) ::: "intel", "volatile");
           // asm!("sub rsp, 8")'

            inner(&mut *(rsp as *mut $crate::arch::interrupts::InterruptErrorStack));

            pop_fs!();
            pop_scratch!();
            pop_preserved!();
            // pop off error code
            asm!("add rsp, 8" :::: "intel", "volatile");
            iretq!();
        }
    };
}

interrupt!(divide_by_zero, stack);
interrupt!(debug, stack);
interrupt!(nonmaskable, stack);
interrupt!(breakpoint, stack);
interrupt!(overflow, stack);
interrupt!(bound_range, stack);
interrupt!(invalid_opcode, stack);
interrupt!(device_not_available, stack);
interrupt_error!(double_fault, stack, {
    //asm!("hlt");
});
interrupt!(coprocessor_segment, stack);

interrupt_error!(invalid_tss, stack);
interrupt_error!(segment_not_present, stack);
interrupt_error!(stack_segment, stack);
interrupt_error!(protection, stack);
interrupt_error!(page, _stack, {
    let cr2: usize;
    asm!("mov rax, cr2" : "={rax}"(cr2) ::: "intel", "volatile");
    println!("Page fault! {}", cr2);
});
interrupt!(fpu, stack);
interrupt_error!(alignment_check, stack);
interrupt!(machine_check, stack);
interrupt!(simd, stack);
interrupt!(virtualization, stack);
interrupt_error!(security, stack);
