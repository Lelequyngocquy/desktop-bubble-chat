pub trait MemoryManager {
    fn trim_working_set(&self, pid: u32);
    fn trim_current_process(&self);
}
