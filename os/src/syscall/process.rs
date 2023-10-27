//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, get_current_status,
        get_current_syscall_times, calculate_run_time, current_user_token, mmap_current_task, unmmap_current_task
    },
    mm::{translated_physaddr, VirtAddr, check_no_mapped, check_all_mapped},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let ts: usize = translated_physaddr(current_user_token(), (_ts as usize).into()).into();
    let ts = ts as *mut TimeVal;
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let ti: usize = translated_physaddr(current_user_token(), (_ti as usize).into()).into();
    let ti = ti as *mut TaskInfo;
    unsafe {
        *ti = TaskInfo {
            status: get_current_status(),
            syscall_times: get_current_syscall_times(),
            time: calculate_run_time(),
        };
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    let vstart: VirtAddr = VirtAddr::from(_start);
    if vstart.page_offset() != 0 {
        error!("start is not aligned by page size");
        -1
    }else if (_port & (!0x7) != 0) || (_port & 0x7 == 0) {
        error!("illegal port");
        -1
    }else if check_no_mapped(current_user_token(), _start, _len) {
        mmap_current_task(vstart, (_start + _len).into(), _port);
        0
    }else {
        error!("There are pages that have been mapped");
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let vstart: VirtAddr = VirtAddr::from(_start);
    if vstart.page_offset() != 0 {
        error!("start is not aligned by page size");
        -1
    } else if check_all_mapped(current_user_token(), _start, _len) {
        if unmmap_current_task(vstart, (_start + _len).into()) {
            0
        }else {
            error!("can't find the memory in areas");
            -1
        }
    } else {
        error!("There are pages that are not mapped");
        -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
