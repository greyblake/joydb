use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

use crate::{JoydbError, Model, Relation, adapters::PartitionedAdapter};

pub trait State: Default + Debug + Serialize + DeserializeOwned + Send + 'static {
    fn is_dirty(&self) -> bool;

    fn reset_dirty(&mut self);

    fn write_with_partitioned_adapter<PA: PartitionedAdapter>(
        &self,
        adapter: &PA,
    ) -> Result<(), crate::JoydbError>;

    fn load_with_partitioned_adapter<PA: PartitionedAdapter>(
        adapter: &PA,
    ) -> Result<Self, JoydbError>;
}

/// A utility trait that implemented by a state that can store a relation of a model.
#[diagnostic::on_unimplemented(
    message = "State `{Self}` does not doest not implement `GetRelation<{M}>`.\nDid you forget to add `{M}` in the state definition?",
    note = "Make sure that model `{M}` is listed in the state definition."
)]
pub trait GetRelation<M: Model> {
    fn get_relation_mut(&mut self) -> &mut Relation<M>;

    fn get_relation(&self) -> &Relation<M>;
}

// TODO: rename define_state -> state! ?
#[macro_export]
macro_rules! define_state {
    (
        $state_type:ident,
        models: [$(
            $model_type:ident
        ),*] $(,)?
    ) => {
        /// A struct that holds the data and can be (de)serialized to/from JSON.
        #[derive(Debug, Default, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(default)]
        #[allow(non_snake_case)]
        pub struct $state_type {
            $(
                $model_type: ::joydb::Relation<$model_type>
            ),+
        }

        impl ::joydb::State for $state_type {
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

            fn write_with_partitioned_adapter<PA: ::joydb::adapters::PartitionedAdapter>(&self, adapter: &PA) -> Result<(), ::joydb::JoydbError> {
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

            fn load_with_partitioned_adapter<PA: ::joydb::adapters::PartitionedAdapter>(adapter: &PA) -> Result<Self, ::joydb::JoydbError> {
                let mut state = Self::default();
                $(
                    state.$model_type = adapter.load_relation::<$model_type>()?;
                )*
                Ok(state)
            }
        }

        $(
            impl ::joydb::GetRelation<$model_type> for $state_type {
                fn get_relation_mut(&mut self) -> &mut ::joydb::Relation<$model_type> {
                    &mut self.$model_type
                }

                fn get_relation(&self) -> &::joydb::Relation<$model_type> {
                    &self.$model_type
                }
            }
        )+
    }
}
