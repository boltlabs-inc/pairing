use std::fmt;
use std::marker::PhantomData;
use hex;
use super::{Fq, FqRepr, Fr, FrRepr, G1Affine, G2Affine, G1, G2};
use {CurveAffine, CurveProjective, EncodedPoint, PrimeField};

use serde::de::{Error as DeserializeError, SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const ERR_CODE: &str = "deserialized bytes do not encode a group element";

impl Serialize for G1 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.into_affine().serialize(s)
    }
}

impl<'de> Deserialize<'de> for G1 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(G1Affine::deserialize(d)?.into_projective())
    }
}

impl Serialize for G1Affine {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serialize_affine(self, s)
    }
}

impl<'de> Deserialize<'de> for G1Affine {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(deserialize_affine(d)?)
    }
}

impl Serialize for G2 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.into_affine().serialize(s)
    }
}

impl<'de> Deserialize<'de> for G2 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(G2Affine::deserialize(d)?.into_projective())
    }
}

impl Serialize for G2Affine {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serialize_affine(self, s)
    }
}

impl<'de> Deserialize<'de> for G2Affine {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(deserialize_affine(d)?)
    }
}

/// Serializes a group element using its compressed representation.
fn serialize_affine<S: Serializer, C: CurveAffine>(c: &C, s: S) -> Result<S::Ok, S::Error> {
    let len = C::Compressed::size();
    let mut w = String::new();
    for byte in c.into_compressed().as_ref() {
        let t = format!("{:02x}", byte);
        w = w + &t;
    }
    s.collect_str(&w)
}

/// Deserializes the compressed representation of a group element.
fn deserialize_affine<'de, D: Deserializer<'de>, C: CurveAffine>(d: D) -> Result<C, D::Error> {
    struct TupleVisitor<C> {
        _ph: PhantomData<C>,
    }

    impl<'de, C: CurveAffine> Visitor<'de> for TupleVisitor<C> {
        type Value = C;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let len = C::Compressed::size();
            write!(f, "a tuple of size {}", len)
        }

        #[inline]
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<C, A::Error> {
            let mut compressed = C::Compressed::empty();
            for (i, byte) in compressed.as_mut().iter_mut().enumerate() {
                let len_err = || DeserializeError::invalid_length(i, &self);
                *byte = seq.next_element()?.ok_or_else(len_err)?;
            }
            let to_err = |_| DeserializeError::custom(ERR_CODE);
            compressed.into_affine().map_err(to_err)
        }

        #[inline]
        fn visit_str<E>(self, v: &str) -> Result<C, E>
        where E: ::serde::de::Error,
        {
            let mut compressed = C::Compressed::empty();
            let _len = C::Compressed::size();
            //let len_err = || DeserializeError::invalid_length(len, &self);
            let w = hex::decode(v).unwrap();
            if (w.len() == _len) {
                for (i, byte) in compressed.as_mut().iter_mut().enumerate() {
                    *byte = w[i];
                }
            }
            let to_err = |_| DeserializeError::custom(ERR_CODE);
            compressed.into_affine().map_err(to_err)
        }

    }

    d.deserialize_str(TupleVisitor{ _ph: PhantomData })
}

impl Serialize for Fr {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.into_repr().serialize(s)
    }
}

impl<'de> Deserialize<'de> for Fr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let fr = FrRepr::deserialize(d)?;
        Fr::from_repr(fr).map_err(|_| D::Error::custom(ERR_CODE))
    }
}

impl Serialize for FrRepr {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> Deserialize<'de> for FrRepr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(FrRepr(<_>::deserialize(d)?))
    }
}

impl Serialize for Fq {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.into_repr().serialize(s)
    }
}

impl<'de> Deserialize<'de> for Fq {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Fq::from_repr(FqRepr::deserialize(d)?).map_err(|_| D::Error::custom(ERR_CODE))
    }
}

impl Serialize for FqRepr {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> Deserialize<'de> for FqRepr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(FqRepr(<_>::deserialize(d)?))
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;

    use super::*;
    use bls12_381::Fq12;

    use std::fmt::Debug;

    use rand::{Rng, SeedableRng, XorShiftRng};

    fn test_roundtrip<T: Serialize + for<'a> Deserialize<'a> + Debug + PartialEq>(t: &T) {
        let ser = serde_json::to_vec(t).unwrap();
        //println!("Bytes: {:?}", ser);
        assert_eq!(*t, serde_json::from_slice(&ser).unwrap());

        let ser2 = serde_json::to_string(t).unwrap();
        //println!("String: {}", ser2);
        assert_eq!(*t, serde_json::from_str(&ser2).unwrap());
    }

    #[test]
    fn serde_g1() {
        let mut rng = XorShiftRng::seed_from_u64(0x5dbe62598d313d76);

        let g: G1 = rng.gen();
        test_roundtrip(&g);
        test_roundtrip(&g.into_affine());
    }

    #[test]
    fn serde_g2() {
        let mut rng = XorShiftRng::seed_from_u64(0x5dbe62598d313d76);
        let g: G2 = rng.gen();
        test_roundtrip(&g);
        test_roundtrip(&g.into_affine());
    }

    #[test]
    fn serde_fr() {
        let mut rng = XorShiftRng::seed_from_u64(0x5dbe62598d313d76);
        let f: Fr = rng.gen();
        test_roundtrip(&f);
        test_roundtrip(&f.into_repr());

    }

    #[test]
    fn serde_fq() {
        let mut rng = XorShiftRng::seed_from_u64(0x5dbe62598d313d76);
        let f: Fq = rng.gen();
        test_roundtrip(&f);
        test_roundtrip(&f.into_repr());
    }

    #[test]
    fn serde_fq12() {
        let mut rng = XorShiftRng::seed_from_u64(0x5dbe62598d313d76);
        let f: Fq12 = rng.gen();
        test_roundtrip(&f);
    }
}
