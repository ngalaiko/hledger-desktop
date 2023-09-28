use tauri::AppHandle;

type StateUpdateFn<T> = Box<dyn Fn(&AppHandle, &mut T)>;

pub enum StateAction<T> {
    Persistent(StateUpdateFn<T>),
    Ephemeral(StateUpdateFn<T>),
}

impl<T> Default for StateAction<T> {
    fn default() -> Self {
        Self::Ephemeral(Box::new(|_, _| {}))
    }
}

impl<T: 'static> StateAction<T> {
    pub fn and_then(self, other: StateAction<T>) -> StateAction<T> {
        match (self, other) {
            (StateAction::Persistent(f1), StateAction::Persistent(f2)) => {
                StateAction::Persistent(Box::new(move |handle, state| {
                    f1(handle, state);
                    f2(handle, state);
                }))
            }
            (StateAction::Persistent(f1), StateAction::Ephemeral(f2)) => {
                StateAction::Persistent(Box::new(move |handle, state| {
                    f1(handle, state);
                    f2(handle, state);
                }))
            }
            (StateAction::Ephemeral(f1), StateAction::Persistent(f2)) => {
                StateAction::Persistent(Box::new(move |handle, state| {
                    f1(handle, state);
                    f2(handle, state);
                }))
            }
            (StateAction::Ephemeral(f1), StateAction::Ephemeral(f2)) => {
                StateAction::Ephemeral(Box::new(move |handle, state| {
                    f1(handle, state);
                    f2(handle, state);
                }))
            }
        }
    }
}
