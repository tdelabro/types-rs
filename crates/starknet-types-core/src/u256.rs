/// Adapter trait for conversion of a type form and into a pair of u128
pub trait StarknetU256: Sized {
    fn from_parts(parts: [u128; 2]) -> Self;
    fn as_parts(&self) -> [u128; 2];
}

#[cfg(feature = "serde")]
pub mod serde {
    use serde::{Deserialize, Deserializer, Serialize};

    use crate::felt::Felt;

    use super::StarknetU256;

    /// Starknet serialization of u256
    ///
    /// Starknet defines here how an u256 should be serialized:
    /// https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/#serialization_in_u256_values
    ///
    /// This method implement a starknet valid way of serializing any types that impl the `StarkentU256` trait
    pub fn serialize_starknet_u256<T, S>(value: &T, ser: S) -> Result<S::Ok, S::Error>
    where
        T: StarknetU256,
        S: serde::Serializer,
    {
        let parts = value.as_parts();

        Serialize::serialize(&[Felt::from(parts[0]), Felt::from(parts[1])], ser)
    }

    /// Starknet deserialization of u256
    ///
    /// Starknet defines here how an u256 should be deserialized:
    /// https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/#serialization_in_u256_values
    ///
    /// This method implement a starknet valid way of deserializing any types that impl the `StarkentU256` trait
    pub fn deserialize_starknet_u256<'de, T, D>(de: D) -> Result<T, D::Error>
    where
        T: StarknetU256,
        D: Deserializer<'de>,
    {
        let parts: [Felt; 2] = Deserialize::deserialize(de)?;
        Ok(T::from_parts([
            parts[0]
                .try_into()
                .map_err(|_| serde::de::Error::custom("invalid u128 value"))?,
            parts[1]
                .try_into()
                .map_err(|_| serde::de::Error::custom("invalid u128 value"))?,
        ]))
    }

    #[cfg(test)]
    pub mod tests {
        use crate::felt::Felt;

        use super::StarknetU256;
        use super::{deserialize_starknet_u256, serialize_starknet_u256};

        #[derive(Debug, PartialEq, Eq)]
        pub struct MyU256 {
            low: u128,
            high: u128,
        }

        #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
        #[serde(transparent)]
        pub struct SerdeWrapper {
            #[serde(
                deserialize_with = "deserialize_starknet_u256",
                serialize_with = "serialize_starknet_u256"
            )]
            inner: MyU256,
        }

        impl StarknetU256 for MyU256 {
            fn from_parts(parts: [u128; 2]) -> Self {
                Self {
                    low: parts[0],
                    high: parts[1],
                }
            }

            fn as_parts(&self) -> [u128; 2] {
                [self.low, self.high]
            }
        }

        const U128_MAX: &str = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

        #[test]
        fn serde_works_as_it_does_for_felts() {
            let low = 0xcafeu128;
            let high = 0xbabeu128;
            let parts = [low, high];

            let v = SerdeWrapper {
                inner: MyU256::from_parts(parts),
            };

            // Serialize
            let serialized_u256 = serde_json::to_string(&v).unwrap();
            let serialized_parts =
                serde_json::to_string(&[Felt::from(low), Felt::from(high)]).unwrap();
            assert_eq!(&serialized_u256, &serialized_parts);

            // Deseraialize
            let deserialize_u256: SerdeWrapper =
                serde_json::from_str(r#"["0xcafe", "0xbabe"]"#).unwrap();
            assert_eq!(deserialize_u256.inner, MyU256 { low, high });

            let serialized_u128_max = format!(r#"["{}","{}"]"#, U128_MAX, U128_MAX);
            let deserialize_u256: SerdeWrapper =
                serde_json::from_str(&serialized_u128_max).unwrap();
            assert_eq!(
                deserialize_u256.inner,
                MyU256 {
                    low: u128::MAX,
                    high: u128::MAX
                }
            );
        }
        #[test]
        fn deserialization_fail_for_values_too_big() {
            let bad_value = format!(r#"["{}1","0x1234"]"#, U128_MAX);
            assert!(serde_json::from_str::<SerdeWrapper>(&bad_value).is_err());
            let bad_value = format!(r#"["0x1234","{}1"]"#, U128_MAX);
            assert!(serde_json::from_str::<SerdeWrapper>(&bad_value).is_err());
        }
    }
}

#[cfg(feature = "primitive-types")]
mod primitive_types {
    use super::StarknetU256;

    impl StarknetU256 for primitive_types::U256 {
        fn from_parts(felts: [u128; 2]) -> Self {
            let mut buffer = [0u8; 32];
            buffer[..16].copy_from_slice(&felts[0].to_le_bytes());
            buffer[16..].copy_from_slice(&felts[1].to_le_bytes());

            primitive_types::U256::from_little_endian(&buffer)
        }
        fn as_parts(&self) -> [u128; 2] {
            let bytes = self.to_little_endian();
            [
                u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
                u128::from_le_bytes(bytes[16..].try_into().unwrap()),
            ]
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::u256::StarknetU256;

        #[test]
        fn works_both_ways() {
            let parts = [0xcafe, 0xbabe];

            let u256 = primitive_types::U256::from_parts(parts);
            assert_eq!(u256.as_parts(), parts);
        }
    }
}
