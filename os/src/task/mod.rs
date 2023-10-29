//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! task(s) for each core.
//!
//! A single global instance of `PID_ALLOCATOR` allocates pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
mod id;
mod manager;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::loader::get_app_data_by_name;
use alloc::sync::Arc;
use lazy_static::*;
pub use manager::{fetch_task, TaskManager};
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};
use crate::mm::{VirtAddr, MapPermission};
use crate::config::MAX_SYSCALL_NUM;
pub use context::TaskContext;
pub use id::{kstack_alloc, pid_alloc, KernelStack, PidHandle};
pub use manager::add_task;
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
    Processor,
};
/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        panic!("All applications completed!");
    }

    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    /// Creation of initial process
    ///
    /// the name "initproc" may be changed to any other app name like "usertests",
    /// but we have user_shell, so we don't need to change it.
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("ch5b_initproc").unwrap()
    ));
}

///Add init process to the manager
pub fn add_initproc() {
    add_task(INITPROC.clone());
}

/// Add the current 'Running' task's syscall times.
pub fn add_current_syscall_times(syscall_id: usize) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.add_syscall_times(syscall_id);
}

/// Calculate the current 'Running' task's total running time
pub fn calculate_run_time() -> usize {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    let us = inner.calculate_run_time();
    ((us / 1_000_000) & 0xffff) * 1000 + (us % 1_000_000) / 1000
}

/// Get the current 'Running' task's task_status.
pub fn get_current_status() -> TaskStatus {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    inner.get_status()
}

/// Get the current 'Running' task's syscall times.
pub fn get_current_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    inner.get_syscall_times()
}


/// Maps a chunk of memory for the current task
pub fn mmap_current_task(start: VirtAddr, end: VirtAddr, flags: usize) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.mmap_task(start, end, flags);
}

/// Unmaps a chunk of memory for the current task
pub fn unmmap_current_task(start: VirtAddr, end: VirtAddr) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.unmap_task(start, end);
}

/// Change current task's priority
pub fn change_current_priority(priority: usize) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.change_priority(priority);
}