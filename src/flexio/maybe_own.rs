pub struct MaybeOwn<T: 'static> {
    inner: MaybeOwnEnum<T>,
}

enum MaybeOwnEnum<T: 'static> {
    Owned(Option<T>),
    StaticRef(&'static T),
}

impl<T: 'static> MaybeOwn<T> {
    pub fn new(t: T) -> Self {
        Self {
            inner: MaybeOwnEnum::Owned(Some(t)),
        }
    }

    pub fn get(&self) -> &T {
        match &self.inner {
            MaybeOwnEnum::Owned(Some(x)) => x,
            MaybeOwnEnum::StaticRef(x) => x,
            MaybeOwnEnum::Owned(None) => unreachable!(),
        }
    }

    pub fn convert_to_static_ref(&mut self, storage: &'static mut Option<T>) -> &'static T {
        match &mut self.inner {
            MaybeOwnEnum::Owned(x) => {
                let x = x.take().unwrap();
                assert!(storage.is_none());
                let x: &'static T = storage.insert(x);
                self.inner = MaybeOwnEnum::StaticRef(x);
                x
            }
            MaybeOwnEnum::StaticRef(x) => x,
        }
    }
}
