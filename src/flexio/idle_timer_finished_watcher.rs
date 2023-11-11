use core::{
    cell::RefCell,
    ops::Deref,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use cortex_m::interrupt::{self, Mutex};
use futures::Future;

struct IdleTimerFinishedWatcherInner<const N: u8> {
    happened: bool,
    waker: Option<Waker>,
    flexio: imxrt_ral::flexio::Instance<N>,
}

pub(crate) struct IdleTimerFinishedWatcher<const N: u8> {
    inner: Mutex<RefCell<IdleTimerFinishedWatcherInner<N>>>,
    idle_timer_id: u8,
}

impl<const N: u8> IdleTimerFinishedWatcherInner<N> {
    pub fn check_and_reset(&mut self, idle_timer_id: u8) {
        let mask = 1u32 << idle_timer_id;
        let flag_set = (imxrt_ral::read_reg!(imxrt_ral::flexio, self.flexio, TIMSTAT) & mask) != 0;

        if flag_set {
            imxrt_ral::write_reg!(imxrt_ral::flexio, self.flexio, TIMSTAT, mask);

            self.happened = true;
            if let Some(waker) = self.waker.take() {
                waker.wake();
            }
        }
    }
}

impl<const N: u8> IdleTimerFinishedWatcher<N> {
    pub fn new(flexio: &imxrt_ral::flexio::Instance<N>, idle_timer_id: u8) -> Self {
        Self {
            inner: Mutex::new(RefCell::new(IdleTimerFinishedWatcherInner {
                happened: false,
                waker: None,
                flexio: unsafe { imxrt_ral::flexio::Instance::<N>::new(flexio.deref()) },
            })),
            idle_timer_id,
        }
    }

    fn with_check_and_reset<R>(
        &self,
        f: impl FnOnce(&mut IdleTimerFinishedWatcherInner<N>) -> R,
    ) -> R {
        interrupt::free(|cs| {
            let inner = self.inner.borrow(cs);
            let mut inner = inner.borrow_mut();
            inner.check_and_reset(self.idle_timer_id);

            f(&mut inner)
        })
    }

    pub fn on_interrupt(&self) {
        self.with_check_and_reset(|_| {});
    }

    pub fn clear(&self) {
        self.with_check_and_reset(|inner| {
            inner.happened = false;
        });
    }

    pub fn poll(&self) -> bool {
        self.with_check_and_reset(|inner| inner.happened)
    }

    pub fn finished(&self) -> IdleTimerFinished<N> {
        IdleTimerFinished(self)
    }
}

pub struct IdleTimerFinished<'a, const N: u8>(&'a IdleTimerFinishedWatcher<N>);

impl<const N: u8> Future for IdleTimerFinished<'_, N> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.with_check_and_reset(|inner| {
            if inner.happened {
                Poll::Ready(())
            } else {
                let new_waker = cx.waker();

                // From embassy
                // https://github.com/embassy-rs/embassy/blob/b99533607ceed225dd12ae73aaa9a0d969a7365e/embassy-sync/src/waitqueue/waker.rs#L59-L61
                match &inner.waker {
                    // Optimization: If both the old and new Wakers wake the same task, we can simply
                    // keep the old waker, skipping the clone. (In most executor implementations,
                    // cloning a waker is somewhat expensive, comparable to cloning an Arc).
                    Some(w2) if (w2.will_wake(new_waker)) => {}
                    _ => {
                        // clone the new waker and store it
                        if let Some(old_waker) = inner.waker.replace(new_waker.clone()) {
                            // We had a waker registered for another task. Wake it, so the other task can
                            // reregister itself if it's still interested.
                            //
                            // If two tasks are waiting on the same thing concurrently, this will cause them
                            // to wake each other in a loop fighting over this WakerRegistration. This wastes
                            // CPU but things will still work.
                            //
                            // If the user wants to have two tasks waiting on the same thing they should use
                            // a more appropriate primitive that can store multiple wakers.
                            old_waker.wake()
                        }
                    }
                }

                Poll::Pending
            }
        })
    }
}
