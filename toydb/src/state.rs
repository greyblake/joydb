use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

pub trait State: Default + Debug + Serialize + DeserializeOwned {
    fn is_dirty(&self) -> bool;

    fn reset_dirty(&mut self);
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
