use core::{cell::UnsafeCell, fmt, mem};
use std::marker::PhantomData;

pub(crate) trait InitFn<Arg>: Sized {
    type Output;
    fn eval(self, arg: Arg) -> Self::Output;
}

pub(crate) struct Susp<T, S, Arg> {
    state: UnsafeCell<State<T, S>>,
    _marker: PhantomData<fn(Arg)>,
}

enum State<T, S> {
    Uninit(S),
    Init(T),
    Poisoned,
}

impl<T, Arg, S: InitFn<Arg, Output = T>> Susp<T, S, Arg> {
    pub(crate) const fn new(suspended: S) -> Self {
        Self {
            state: UnsafeCell::new(State::Uninit(suspended)),
            _marker: PhantomData,
        }
    }

    pub(crate) const fn init(value: T) -> Self {
        Self {
            state: UnsafeCell::new(State::Init(value)),
            _marker: PhantomData,
        }
    }

    pub(crate) fn force(&self, arg: Arg) -> &T {
        let state = unsafe { &*self.state.get() };
        match state {
            State::Uninit(_) => unsafe { self.really_init(arg) },
            State::Init(val) => val,
            State::Poisoned => panic_poisoned(),
        }
    }

    #[cold]
    unsafe fn really_init(&self, arg: Arg) -> &T {
        let state = unsafe { &mut *self.state.get() };
        let State::Uninit(s) = mem::replace(state, State::Poisoned) else {
            unreachable!();
        };
        let val = s.eval(arg);
        // stateはすでに無効であることに注意
        unsafe {
            self.state.get().write(State::Init(val));
        }
        let state = unsafe { &*self.state.get() };
        let State::Init(val) = state else {
            unreachable!();
        };
        val
    }
}

impl<T: fmt::Debug, S, A> fmt::Debug for Susp<T, S, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("LazyCell");
        let state = unsafe { &*self.state.get() };
        match state {
            State::Init(data) => d.field(data),
            State::Uninit(_) => d.field(&format_args!("<uninit>")),
            State::Poisoned => d.field(&format_args!("<poisoned>")),
        };
        d.finish()
    }
}

#[cold]
#[inline(never)]
const fn panic_poisoned() -> ! {
    panic!("LazyCell instance has previously been poisoned")
}
