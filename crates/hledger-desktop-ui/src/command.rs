type UpdateFn<'cmd, T> = Box<dyn FnOnce(&mut T) + 'cmd>;

pub enum Command<'cmd, T: 'cmd> {
    Persistent(UpdateFn<'cmd, T>),
    Ephemeral(UpdateFn<'cmd, T>),
}

impl<'cmd, T> Command<'cmd, T> {
    pub fn persistent(update: impl FnOnce(&mut T) + 'cmd) -> Self {
        Self::Persistent(Box::new(update))
    }

    pub fn ephemeral(update: impl FnOnce(&mut T) + 'cmd) -> Self {
        Self::Ephemeral(Box::new(update))
    }
}

impl<T> Default for Command<'_, T> {
    fn default() -> Self {
        Self::Ephemeral(Box::new(|_| {}))
    }
}

impl<'cmd, T> Command<'cmd, T> {
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    pub fn map<O>(
        self,
        update: impl FnOnce(UpdateFn<T>) -> UpdateFn<O> + 'cmd,
    ) -> Command<'cmd, O> {
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
    pub fn and_then(self, other: Command<'cmd, T>) -> Command<'cmd, T> {
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
