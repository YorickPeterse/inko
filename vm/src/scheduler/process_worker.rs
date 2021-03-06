//! Executing of lightweight Inko processes in a single thread.
use crate::arc_without_weak::ArcWithoutWeak;
use crate::gc::tracer::Pool as TracerPool;
use crate::process::RcProcess;
use crate::scheduler::pool_state::PoolState;
use crate::scheduler::queue::RcQueue;
use crate::scheduler::worker::Worker;
use crate::vm::machine::Machine;
use std::cell::UnsafeCell;

/// The state that a worker is in.
#[derive(Eq, PartialEq, Debug)]
pub enum Mode {
    /// The worker should process its own queue or other queues in a normal
    /// fashion.
    Normal,

    /// The worker should only process a particular job, and not steal any other
    /// jobs.
    Exclusive,
}

/// A worker owned by a thread, used for executing jobs from a scheduler queue.
pub struct ProcessWorker {
    /// The unique ID of this worker, used for pinning jobs.
    pub id: usize,

    /// The thread pool used for tracing live objects during garbage collection.
    pub tracers: TracerPool,

    /// The queue owned by this worker.
    queue: RcQueue<RcProcess>,

    /// The state of the pool this worker belongs to.
    state: ArcWithoutWeak<PoolState<RcProcess>>,

    /// The Machine to use for running code.
    machine: UnsafeCell<Machine>,

    /// The mode this worker is in.
    mode: Mode,
}

impl ProcessWorker {
    /// Starts a new worker operating in the normal mode.
    pub fn new(
        id: usize,
        queue: RcQueue<RcProcess>,
        state: ArcWithoutWeak<PoolState<RcProcess>>,
        machine: Machine,
    ) -> Self {
        let tracers = machine.state.config.tracer_threads;

        ProcessWorker {
            id,
            queue,
            state,
            mode: Mode::Normal,
            machine: UnsafeCell::new(machine),
            tracers: TracerPool::new(tracers),
        }
    }

    /// Changes the worker state so it operates in exclusive mode.
    ///
    /// When in exclusive mode, only the currently running job will be allowed
    /// to run on this worker. All other jobs are pushed back into the global
    /// queue.
    pub fn enter_exclusive_mode(&mut self) {
        self.queue.move_external_jobs();

        while let Some(job) = self.queue.pop() {
            self.state.push_global(job);
        }

        self.mode = Mode::Exclusive;
    }

    pub fn leave_exclusive_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    /// Performs a single iteration of the normal work loop.
    fn normal_iteration(&mut self) {
        if self.process_local_jobs() {
            return;
        }

        if self.steal_from_other_queue() {
            return;
        }

        if self.queue.move_external_jobs() {
            return;
        }

        if self.steal_from_global_queue() {
            return;
        }

        self.state.park_while(|| {
            !self.state.has_global_jobs() && !self.queue.has_external_jobs()
        });
    }

    /// Runs a single iteration of an exclusive work loop.
    fn exclusive_iteration(&mut self) {
        if self.process_local_jobs() {
            return;
        }

        // Moving external jobs would allow other workers to steal them,
        // starving the current worker of pinned jobs. Since only one job can be
        // pinned to a worker, we don't need a loop here.
        if let Some(job) = self.queue.pop_external_job() {
            self.process_job(job);
            return;
        }

        self.state.park_while(|| !self.queue.has_external_jobs());
    }
}

impl Worker<RcProcess> for ProcessWorker {
    fn state(&self) -> &PoolState<RcProcess> {
        &self.state
    }

    fn queue(&self) -> &RcQueue<RcProcess> {
        &self.queue
    }

    fn run(&mut self) {
        while self.state.is_alive() {
            match self.mode {
                Mode::Normal => self.normal_iteration(),
                Mode::Exclusive => self.exclusive_iteration(),
            };
        }
    }

    fn process_job(&mut self, job: RcProcess) {
        // When using a Machine we need both an immutable reference to it (using
        // `self.machine`), and a mutable reference to pass as an argument.
        // Rust does not allow this, even though in this case it's perfectly
        // safe.
        //
        // To work around this we use UnsafeCell. We could use RefCell, but
        // since we know exactly how this code is used (it's only the lines
        // below that depend on this) the runtime reference counting is not
        // needed.
        let machine = unsafe { &mut *self.machine.get() };

        machine.run(self, &job);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::instructions::process;
    use crate::vm::test::setup;

    fn worker(machine: Machine) -> ProcessWorker {
        let pool_state = machine.state.scheduler.primary_pool.state.clone();
        let queue = pool_state.queues[0].clone();

        ProcessWorker::new(0, queue, pool_state, machine)
    }

    #[test]
    fn test_run_global_jobs() {
        let (machine, _block, process) = setup();
        let mut worker = worker(machine.clone());

        worker.state.push_global(process.clone());
        worker.run();

        assert!(worker.state.pop_global().is_none());
        assert_eq!(worker.state.queues[0].has_local_jobs(), false);
    }

    #[test]
    fn test_run_with_external_jobs() {
        let (machine, _block, process) = setup();
        let mut worker = worker(machine.clone());

        worker.state.queues[0].push_external(process.clone());
        worker.run();

        assert_eq!(worker.state.queues[0].has_external_jobs(), false);
    }

    #[test]
    fn test_run_steal_then_terminate() {
        let (machine, _block, process) = setup();
        let mut worker = worker(machine.clone());

        worker.state.queues[1].push_internal(process.clone());
        worker.run();

        assert_eq!(worker.state.queues[1].has_local_jobs(), false);
    }

    #[test]
    fn test_run_work_and_steal() {
        let (machine, block, process) = setup();
        let process2 = process::process_allocate(&machine.state, &block);
        let mut worker = worker(machine.clone());

        worker.queue.push_internal(process2);
        worker.state.queues[1].push_internal(process);

        // Here the order of work is:
        //
        // 1. Process local job
        // 2. Steal from other queue
        // 3. Terminate
        worker.run();

        assert_eq!(worker.queue.has_local_jobs(), false);
        assert_eq!(worker.state.queues[1].has_local_jobs(), false);
    }

    #[test]
    fn test_run_work_then_terminate_steal_loop() {
        let (machine, block, process) = setup();
        let process2 = process::process_allocate(&machine.state, &block);
        let mut worker = worker(machine.clone());

        worker.state.queues[0].push_internal(process);
        worker.state.queues[1].push_internal(process2);
        worker.run();

        assert_eq!(worker.state.queues[0].has_local_jobs(), false);
        assert!(worker.state.queues[1].has_local_jobs());
    }

    #[test]
    fn test_run_exclusive_iteration() {
        let (machine, _block, process) = setup();
        let mut worker = worker(machine.clone());

        worker.enter_exclusive_mode();
        worker.queue.push_external(process);
        worker.run();

        assert_eq!(worker.queue.has_external_jobs(), false);
    }

    #[test]
    fn test_enter_exclusive_mode() {
        let (machine, block, process) = setup();
        let process2 = process::process_allocate(&machine.state, &block);
        let mut worker = worker(machine.clone());

        worker.queue.push_internal(process);
        worker.queue.push_external(process2);
        worker.enter_exclusive_mode();

        assert_eq!(worker.mode, Mode::Exclusive);
        assert_eq!(worker.queue.has_local_jobs(), false);
        assert!(worker.queue.pop_external_job().is_none());
    }

    #[test]
    fn test_leave_exclusive_mode() {
        let (machine, _block, _process) = setup();
        let mut worker = worker(machine.clone());

        worker.enter_exclusive_mode();
        worker.leave_exclusive_mode();

        assert_eq!(worker.mode, Mode::Normal);
    }
}
