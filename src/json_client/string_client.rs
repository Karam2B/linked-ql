    use crate::{
        gen_serde::{
            ObjectEncoding, Serialize, json_format_side::PartialDeserialize,
        },
        json_client::client_interface::Client,
        sub_arc::ArcSubStr,
    };

    pub struct StringClient {
        pub(super) inner: Client,
    }

    impl Client {
        pub fn into_string_client(self) -> StringClient {
            StringClient { inner: self }
        }
    }

    pub(super) struct StringClientInput {
        pub(super) op: ArcSubStr,
        pub(super) body: PartialDeserialize,
    }

    impl crate::gen_serde::DeserializeSpec for StringClientInput {
        type Meta = ();
    }

    pub(super) struct StringClientOutput<T> {
        pub(super) output: T,
    }

    impl<F, T> Serialize<F> for StringClientOutput<T>
    where
        F: ObjectEncoding,
        T: Serialize<F>,
        str: Serialize<F>,
    {
        fn serialize(&self, ctx: &mut F) {
            let mut object = ctx.serialize_start();
            ctx.serialize_pair(&mut object, "output", &self.output);
            ctx.serialize_end(object);
        }
    }

    impl<'de, S> crate::gen_serde::Deserialize<'de, S> for StringClientInput
    where
        S: crate::gen_serde::Deserializer<'de>,
        ArcSubStr: crate::gen_serde::Deserialize<'de, S>,
        PartialDeserialize: crate::gen_serde::Deserialize<'de, S>,
        S: crate::gen_serde::DeserializeMap<'de>,
        S: crate::gen_serde::KnownKey<&'static str>,
    {
        fn deserialize(_meta: Self::Meta, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = crate::gen_serde::DeserializeMap::start_map(serialized)?;
            let op = crate::gen_serde::DeserializeMap::deserialize_with_known_key(
                serialized,
                &mut map,
                "op",
                (),
            )?;
            let body = crate::gen_serde::DeserializeMap::deserialize_with_known_key(
                serialized,
                &mut map,
                "body",
                (),
            )?;
            crate::gen_serde::DeserializeMap::finish(serialized, map)?;
            Ok(StringClientInput { op, body })
        }
    }
