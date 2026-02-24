mod entity;
mod grouped;

pub(crate) use entity::EntityState;
pub(crate) use grouped::GroupedState;

#[derive(Debug, thiserror::Error)]
pub(crate) enum StateError {
    #[error("state lock poisoned: state={state}, lock={lock}")]
    LockPoisoned {
        state: &'static str,
        lock: &'static str,
    },
}
