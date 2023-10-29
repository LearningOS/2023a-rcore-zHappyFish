//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

const BIG_STRIDE: usize = 0x8000000;

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut min_stride = None;
        let mut index = None;
        for (i, task) in self.ready_queue.iter().enumerate() {
            let inner = task.inner_exclusive_access();
            if let Some(min) = min_stride {
                if inner.stride < min {
                    index = Some(i);
                    min_stride = Some(inner.stride);
                }
            }else {
                index = Some(i);
                min_stride = Some(inner.stride);
            }
        }
        if let Some(i) = index {
            let task = self.ready_queue.remove(i).unwrap();
            let mut inner = task.inner_exclusive_access();
            inner.stride += BIG_STRIDE / inner.priority;
            drop(inner);
            Some(task)
        }
        else {
            None
        }
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
