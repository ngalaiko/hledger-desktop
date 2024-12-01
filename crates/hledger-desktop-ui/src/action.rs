type UpdateFn<T> = Box<dyn FnOnce(&mut T)>;

pub enum Action<T> {
    Persistent(UpdateFn<T>),
    Ephemeral(UpdateFn<T>),
}

impl<T> Default for Action<T> {
    fn default() -> Self {
        Self::Ephemeral(Box::new(|_| {}))
    }
}

impl<T: 'static> Action<T> {
    #[must_use]
    pub fn noop() -> Self {
        Self::default()
    }

    pub fn map<O: 'static>(
        self,
        update: impl FnOnce(UpdateFn<T>) -> UpdateFn<O> + 'static,
    ) -> Action<O> {
        match self {
            Self::Ephemeral(t) => Action::<O>::Ephemeral(Box::new(|o: &mut O| {
                update(t)(o);
            })),
            Self::Persistent(t) => Action::<O>::Persistent(Box::new(|o: &mut O| {
                update(t)(o);
            })),
        }
    }

    #[must_use]
    pub fn and_then(self, other: Action<T>) -> Action<T> {
        match (self, other) {
            (Action::Persistent(f1), Action::Persistent(f2)) => {
                Action::Persistent(Box::new(move |state| {
                    f1(state);
                    f2(state);
                }))
            }
            (Action::Persistent(f1), Action::Ephemeral(f2)) => {
                Action::Persistent(Box::new(move |state| {
                    f1(state);
                    f2(state);
                }))
            }
            (Action::Ephemeral(f1), Action::Persistent(f2)) => {
                Action::Persistent(Box::new(move |state| {
                    f1(state);
                    f2(state);
                }))
            }
            (Action::Ephemeral(f1), Action::Ephemeral(f2)) => {
                Action::Ephemeral(Box::new(move |state| {
                    f1(state);
                    f2(state);
                }))
            }
        }
    }
}
