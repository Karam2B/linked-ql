#[cfg(test)]
mod tests {
    use quote::quote;

    #[test]
    fn test_data_flow() {
        let input = quote!(
            fn impl_link<Key, From, To>(
                this: SetNew<OneToMany<Key, From, To>, To::InputData>,
            ) where
                Key: Clone,
                From: Clone,
                To: Clone,
                To: CollectionMeta,
                <To::Id as CollectionId>::IdData: Clone,
            {
                let meta = use meta(self.relation);

                let insert_new_one = use pre_op(InsertOne {
                    base: meta.clone(),
                    data: AutoGenerate,
                    data: this.data,
                    links: (),
                });

                use insert_values(Bind(insert_new_one.id.clone()));

                use output(insert_new_one);
            }
        );

        let output = quote!(
            fn impl_link<Key, From, To>(this: SetNew<OneToMany<Key, From, To>, To::InputData>)
            where
                Key: Clone,
                From: Clone,
                To: Clone,
                To: CollectionMeta,
                <To::Id as CollectionId>::IdData: Clone,
            {
                InsertLinkFromClosures {
                    meta: move || self.relation,
                    pre_op: move |meta| InsertOne {
                        base: meta.clone(),
                        data: AutoGenerate,
                        data: this.data,
                        links: (),
                    },
                    insert_values: move |meta, pre_op| Bind(pre_op.id.clone()),
                    output: move |meta, pre_op| pre_op,
                }
            }
        );

        pretty_assertions::assert_eq!(input.to_string(), output.to_string());
    }
}
