#[macro_export]
macro_rules! define_state {
    (
        $state_type:ident,
        $(
            $plural:ident : $model_type:ty
        ),* $(,)?
    ) => {
        /// A struct that holds the data and can be (de)serialized to/from JSON.
        #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
        #[serde(default)]
        pub struct $state_type {
            $(
                $plural: Vec<$model_type>
            ),+
        }

        $(
            impl ::toydb::GetRelation<$model_type> for $state_type {
                fn get_rel_mut(&mut self) -> &mut Vec<$model_type> {
                    &mut self.$plural
                }

                fn get_rel(&self) -> &Vec<$model_type> {
                    &self.$plural
                }
            }
        )+
    }
}
