use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

use crate::{
    Model, Relation, ToydbError,
    adapters::{Adapter, PartitionedAdapter},
};

pub trait State: Default + Debug + Serialize + DeserializeOwned {
    fn is_dirty(&self) -> bool;

    fn reset_dirty(&mut self);

    // TODO: Remove this method and inline it in db.rs
    fn write_with_adapter<A: Adapter>(&self, adapter: &A) -> Result<(), crate::ToydbError> {
        adapter.write_state(self)
    }

    fn write_with_partitioned_adapter<PA: PartitionedAdapter>(
        &self,
        adapter: &PA,
    ) -> Result<(), crate::ToydbError>;

    fn load_with_partitioned_adapter<PA: PartitionedAdapter>(
        adapter: &PA,
    ) -> Result<Self, ToydbError>;

    fn init_with_partitioned_adapter<PA: PartitionedAdapter>(
        adapter: &PA,
    ) -> Result<Self, ToydbError>;
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

            fn write_with_partitioned_adapter<PA: ::toydb::adapters::PartitionedAdapter>(&self, adapter: &PA) -> Result<(), ::toydb::ToydbError> {
                $(
                    {
                        let relation = &self.$model_type;
                        if relation.is_dirty() {
                            adapter.write_relation(relation)?;
                        }
                    }
                )*
                Ok(())
            }

            fn load_with_partitioned_adapter<PA: ::toydb::adapters::PartitionedAdapter>(adapter: &PA) -> Result<Self, ::toydb::ToydbError> {
                let mut state = Self::default();
                $(
                    state.$model_type = adapter.read_relation::<$model_type>()?;
                )*
                Ok(state)
            }

            fn init_with_partitioned_adapter<PA: ::toydb::adapters::PartitionedAdapter>(adapter: &PA) -> Result<Self, ::toydb::ToydbError> {
                let mut state = Self::default();
                $(
                    state.$model_type = adapter.init_relation::<$model_type>()?;
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
