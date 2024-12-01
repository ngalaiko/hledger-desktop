type UpdateFn<T> = Box<dyn FnOnce(&mut T)>;

pub enum Command<T> {
    Persistent(UpdateFn<T>),
    Ephemeral(UpdateFn<T>),
}

impl<T> Command<T> {
    pub fn persistent(update: impl FnOnce(&mut T) + 'static) -> Self {
        Self::Persistent(Box::new(update))
    }

    pub fn ephemeral(update: impl FnOnce(&mut T) + 'static) -> Self {
        Self::Ephemeral(Box::new(update))
    }
}

impl<T> Default for Command<T> {
    fn default() -> Self {
        Self::Ephemeral(Box::new(|_| {}))
    }
}

impl<T: 'static> Command<T> {
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    pub fn map<O: 'static>(
        self,
        update: impl FnOnce(UpdateFn<T>) -> UpdateFn<O> + 'static,
    ) -> Command<O> {
        match self {
            Self::Ephemeral(t) => Command::<O>::Ephemeral(Box::new(|o: &mut O| {
                update(t)(o);
            })),
            Self::Persistent(t) => Command::<O>::Persistent(Box::new(|o: &mut O| {
                update(t)(o);
            })),
        }
    }

    #[must_use]
    pub fn and_then(self, other: Command<T>) -> Command<T> {
        match (self, other) {
            (Command::Persistent(f1), Command::Persistent(f2)) => {
                Command::Persistent(Box::new(move |state| {
                    f1(state);
                    f2(state);
                }))
            }
            (Command::Persistent(f1), Command::Ephemeral(f2)) => {
                Command::Persistent(Box::new(move |state| {
                    f1(state);
                    f2(state);
                }))
            }
            (Command::Ephemeral(f1), Command::Persistent(f2)) => {
                Command::Persistent(Box::new(move |state| {
                    f1(state);
                    f2(state);
                }))
            }
            (Command::Ephemeral(f1), Command::Ephemeral(f2)) => {
                Command::Ephemeral(Box::new(move |state| {
                    f1(state);
                    f2(state);
                }))
            }
        }
    }
}
