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
                $model_type: Vec<$model_type>
            ),+
        }

        impl ::toydb::State for $state_type {
            type PlainState = Self;

            fn to_plain(&self) -> Self::PlainState {
                // TODO: Introduce something like PlainStateRef to avoid cloning
                // Example:
                //    struct PlainStateRef<'a> {
                //        users: &'a Vec<User>,
                //        posts: &'a Vec<Post>,
                //        // etc
                //    }
                //
                Self {
                    $(
                        $model_type: self.$model_type.clone()
                    ),+
                }
            }

            fn from_plain(plain: Self::PlainState) -> Self {
                plain
            }
        }

        $(
            impl ::toydb::GetRelation<$model_type> for $state_type {
                fn get_rel_mut(&mut self) -> &mut Vec<$model_type> {
                    &mut self.$model_type
                }

                fn get_rel(&self) -> &Vec<$model_type> {
                    &self.$model_type
                }
            }
        )+
    }
}
