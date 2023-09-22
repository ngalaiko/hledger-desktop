use tauri::AppHandle;

pub type StateUpdateFn<T> = Box<dyn Fn(&AppHandle, &mut T)>;

pub enum StateUpdate<T> {
    Persistent(StateUpdateFn<T>),
    Ephemeral(StateUpdateFn<T>),
}

impl<T: 'static> StateUpdate<T> {
    pub fn and_then(self, other: StateUpdate<T>) -> StateUpdate<T> {
        match (self, other) {
            (StateUpdate::Persistent(f1), StateUpdate::Persistent(f2)) => {
                StateUpdate::Persistent(Box::new(move |handle, state| {
                    f1(handle, state);
                    f2(handle, state);
                }))
            }
            (StateUpdate::Persistent(f1), StateUpdate::Ephemeral(f2)) => {
                StateUpdate::Persistent(Box::new(move |handle, state| {
                    f1(handle, state);
                    f2(handle, state);
                }))
            }
            (StateUpdate::Ephemeral(f1), StateUpdate::Persistent(f2)) => {
                StateUpdate::Persistent(Box::new(move |handle, state| {
                    f1(handle, state);
                    f2(handle, state);
                }))
            }
            (StateUpdate::Ephemeral(f1), StateUpdate::Ephemeral(f2)) => {
                StateUpdate::Ephemeral(Box::new(move |handle, state| {
                    f1(handle, state);
                    f2(handle, state);
                }))
            }
        }
    }
}
