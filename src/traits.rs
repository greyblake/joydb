use std::fmt::{Debug, Display};

/// An abstraction that allows to identify an entity by its id.
pub trait Identifiable {
    type Id: Debug + Display + Clone + Copy + Eq;

    fn id(&self) -> Self::Id;
}

// Implement Identifiable traits with the following assumptions:
// * Identifiable is imported
// * A field that carries Id is `id`.
//
// Example:
//   impl_identifiable!(User, UserId);
#[macro_export(local_inner_macros)]
macro_rules! impl_identifiable {
    ($tp:ident, $id_tp:ident) => {
        impl crate::traits::Identifiable for $tp {
            type Id = $id_tp;

            fn id(&self) -> $id_tp {
                self.id
            }
        }
    };
}
