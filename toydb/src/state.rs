use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

use crate::{
    Model, Relation,
    adapters::{Backend, RelationAdapter, UnifiedAdapter},
};

pub trait State: Default + Debug + Serialize + DeserializeOwned {
    fn is_dirty(&self) -> bool;

    fn reset_dirty(&mut self);

    fn write_with_backend<UA: UnifiedAdapter, RA: RelationAdapter>(
        &self,
        backend: &Backend<UA, RA>,
        path: &std::path::Path,
    ) -> Result<(), crate::ToydbError> {
        match backend {
            Backend::Unified(unified_adapter) => unified_adapter.write(self),
            Backend::Partitioned(_adapter) => self.write_relations::<RA>(path),
        }
    }

    fn write_relations<RA: RelationAdapter>(
        &self,
        base_path: &std::path::Path,
    ) -> Result<(), crate::ToydbError>;

    fn load_with_backend<UA: UnifiedAdapter, RA: RelationAdapter>(
        backend: &Backend<UA, RA>,
        path: &std::path::Path,
    ) -> Result<Self, crate::ToydbError> {
        match backend {
            Backend::Unified(unified_adapter) => unified_adapter.read(),
            Backend::Partitioned(_adapter) => Self::load_relations::<RA>(path),
        }
    }

    fn load_relations<RA: RelationAdapter>(
        base_path: &std::path::Path,
    ) -> Result<Self, crate::ToydbError>;
}

/// A utility trait that implemented by a state that can store a relation of a model.
#[diagnostic::on_unimplemented(
    message = "State `{Self}` does not doest not implement `GetRelation<{M}>`.\nDid you forget to add `{M}` in the state definition?",
    note = "Make sure that model `{M}` is listed in the state definition."
)]
pub trait GetRelation<M: Model> {
    fn get_rel_mut(&mut self) -> &mut Relation<M>;

    fn get_rel(&self) -> &Relation<M>;
}

#[macro_export]
macro_rules! define_state {
    (
        $state_type:ident,
        models: [$(
            $model_type:ident
        ),*] $(,)?
    ) => {
        /// A struct that holds the data and can be (de)serialized to/from JSON.
        #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
        #[serde(default)]
        #[allow(non_snake_case)]
        pub struct $state_type {
            $(
                $model_type: ::toydb::Relation<$model_type>
            ),+
        }

        impl ::toydb::State for $state_type {
            fn is_dirty(&self) -> bool {
                $(
                    self.$model_type.is_dirty()
                )||+
            }

            fn reset_dirty(&mut self) {
                $(
                    self.$model_type.reset_dirty();
                )*
            }

            fn write_relations<RA: ::toydb::RelationAdapter>(&self, base_path: &std::path::Path) -> Result<(), ::toydb::ToydbError> {
                $(
                    {
                        let relation = &self.$model_type;
                        if relation.is_dirty() {
                            RA::write(base_path, relation)?;
                        }
                    }
                )*
                Ok(())
            }

            fn load_relations<RA: ::toydb::RelationAdapter>(base_path: &std::path::Path) -> Result<Self, ::toydb::ToydbError> {
                let mut state = Self::default();
                $(
                    state.$model_type = RA::read::<$model_type>(base_path)?;
                )*
                Ok(state)
            }
        }

        $(
            impl ::toydb::GetRelation<$model_type> for $state_type {
                fn get_rel_mut(&mut self) -> &mut ::toydb::Relation<$model_type> {
                    &mut self.$model_type
                }

                fn get_rel(&self) -> &::toydb::Relation<$model_type> {
                    &self.$model_type
                }
            }
        )+
    }
}
