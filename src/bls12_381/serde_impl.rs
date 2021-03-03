use super::{Fq, FqRepr, Fq12, Fq6, Fq2, Fr, FrRepr, G1Affine, G2Affine, G1, G2};
use hex;
use std::fmt;
use std::marker::PhantomData;
use {CurveAffine, CurveProjective, EncodedPoint, PrimeField};

use serde::de::{Error as DeserializeError, SeqAccess, Visitor};
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
    let _len = C::Compressed::size();
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
            where
                E: ::serde::de::Error,
        {
            let mut compressed = C::Compressed::empty();
            let _len = C::Compressed::size();
            //let len_err = || DeserializeError::invalid_length(len, &self);
            let w = hex::decode(v).unwrap();
            if w.len() == _len {
                for (i, byte) in compressed.as_mut().iter_mut().enumerate() {
                    *byte = w[i];
                }
            }
            let to_err = |_| DeserializeError::custom(ERR_CODE);
            compressed.into_affine().map_err(to_err)
        }
    }

    d.deserialize_str(TupleVisitor { _ph: PhantomData })
}

fn transform_u64_to_array_of_u8(x: u64) -> [u8; 8] {
    let b1: u8 = ((x >> 56) & 0xff) as u8;
    let b2: u8 = ((x >> 48) & 0xff) as u8;
    let b3: u8 = ((x >> 40) & 0xff) as u8;
    let b4: u8 = ((x >> 32) & 0xff) as u8;
    let b5: u8 = ((x >> 24) & 0xff) as u8;
    let b6: u8 = ((x >> 16) & 0xff) as u8;
    let b7: u8 = ((x >> 8) & 0xff) as u8;
    let b8: u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4, b5, b6, b7, b8];
}

fn transform_bytes_to_u64(x: &Vec<u8>) -> u64 {
    let mut u: u64 = 0;
    let len = x.len() - 1;
    for i in 0..8 {
        let t: u64 = (x[len - i] as u64) << (i * 8);
        u += t;
    }
    return u;
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
        // self.0.serialize(s)
        let r = self.0;
        let mut v = String::new();
        for byte in r.as_ref() {
            let mut byte_array = transform_u64_to_array_of_u8(*byte);
            byte_array.reverse(); // preserve little endian encoding
            let hex_str = hex::encode(&byte_array);
            v += &hex_str;
        }
        s.serialize_str(&v)
    }
}

impl<'de> Deserialize<'de> for FrRepr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct FrReprTupleVisitor;

        impl<'de> Visitor<'de> for FrReprTupleVisitor {
            type Value = FrRepr;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "sequence of bytes representing FrRepr element")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where E: ::serde::de::Error,
            {
                match v.len() == 64 {
                    true => {
                        let mut bytes: Vec<u64> = Vec::new();
                        let str_tmp = [
                            hex::decode(&v[0..16]).to_owned(),
                            hex::decode(&v[16..32]).to_owned(),
                            hex::decode(&v[32..48]).to_owned(),
                            hex::decode(&v[48..64]).to_owned(),
                        ];
                        for bb in str_tmp.iter() {
                            if bb.is_ok() {
                                let mut c = bb.as_ref().unwrap().clone();
                                c.reverse(); 
                                bytes.push(transform_bytes_to_u64(&c));
                            }
                        }

                        let mut byte_slice: [u64; 4] = [0; 4];
                        if bytes.len() == 4 {
                            // let to_err = |_| DeserializeError::custom(ERR_CODE);
                            byte_slice.copy_from_slice(&bytes[0..4]);
                        }
                        Ok(FrRepr(byte_slice))
                    }
                    false => return Err(serde::de::Error::custom("invalid length: expected 64 bytes")),
                }
            }
        }
        d.deserialize_str(FrReprTupleVisitor {})
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
        // self.0.serialize(s)
        let r = self.0;
        let mut v = String::new();
        for byte in r.as_ref() {
            let byte_array = transform_u64_to_array_of_u8(*byte);
            let hex_str = hex::encode(&byte_array);
            v += &hex_str;
        }
        s.serialize_str(&v)
    }
}

impl<'de> Deserialize<'de> for FqRepr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct FqReprTupleVisitor;

        impl<'de> Visitor<'de> for FqReprTupleVisitor {
            type Value = FqRepr;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "sequence of bytes representing FqRepr element")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where E: ::serde::de::Error,
            {
                match v.len() == 96 {
                    true => {
                        let mut bytes: Vec<u64> = Vec::new();
                        let str_tmp = [
                            hex::decode(&v[0..16]).to_owned(),
                            hex::decode(&v[16..32]).to_owned(),
                            hex::decode(&v[32..48]).to_owned(),
                            hex::decode(&v[48..64]).to_owned(),
                            hex::decode(&v[64..80]).to_owned(),
                            hex::decode(&v[80..96]).to_owned(),
                        ];
                        for bb in str_tmp.iter() {
                            if bb.is_ok() {
                                let c = bb.as_ref().unwrap().clone();
                                bytes.push(transform_bytes_to_u64(&c));
                            }
                        }

                        let mut byte_slice: [u64; 6] = [0; 6];
                        if bytes.len() == 6 {
                            // let to_err = |_| DeserializeError::custom(ERR_CODE);
                            byte_slice.copy_from_slice(&bytes[0..6]);
                        }
                        Ok(FqRepr(byte_slice))
                    }
                    false => return Err(serde::de::Error::custom("invalid length: expected 96 bytes")),
                }
            }
        }
        d.deserialize_str(FqReprTupleVisitor {})
    }
}

impl Serialize for Fq12 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let string121 = serialize_fq_repr(self.c1.c2.c1.into_repr()).to_owned();
        let string120 = serialize_fq_repr(self.c1.c2.c0.into_repr());
        let string111 = serialize_fq_repr(self.c1.c1.c1.into_repr());
        let string110 = serialize_fq_repr(self.c1.c1.c0.into_repr());
        let string101 = serialize_fq_repr(self.c1.c0.c1.into_repr());
        let string100 = serialize_fq_repr(self.c1.c0.c0.into_repr());
        let string021 = serialize_fq_repr(self.c0.c2.c1.into_repr());
        let string020 = serialize_fq_repr(self.c0.c2.c0.into_repr());
        let string011 = serialize_fq_repr(self.c0.c1.c1.into_repr());
        let string010 = serialize_fq_repr(self.c0.c1.c0.into_repr());
        let string001 = serialize_fq_repr(self.c0.c0.c1.into_repr());
        let string000 = serialize_fq_repr(self.c0.c0.c0.into_repr());
        let string = string121 + string120.as_str() + string111.as_str() + string110.as_str()
            + string101.as_str() + string100.as_str() + string021.as_str()
            + string020.as_str() + string011.as_str() + string010.as_str()
            + string001.as_str() + string000.as_str();
        s.serialize_str(&string)
    }
}

fn serialize_fq_repr(r: FqRepr) -> String {
    let mut out = format!("{}", r);
    out.remove(0);
    out.remove(0);
    out
}

impl<'de> Deserialize<'de> for Fq12 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Fq12StringVisitor;

        impl<'de> Visitor<'de> for Fq12StringVisitor {
            type Value = Fq12;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "sequence of bytes representing Fq12 element")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where E: ::serde::de::Error,
            {
                match v.len() == 1152 {
                    true => {
                        let c121 = deserialize_fq(&v[..96]);
                        let c120 = deserialize_fq(&v[96..192]);
                        let c111 = deserialize_fq(&v[192..288]);
                        let c110 = deserialize_fq(&v[288..384]);
                        let c101 = deserialize_fq(&v[384..480]);
                        let c100 = deserialize_fq(&v[480..576]);
                        let c021 = deserialize_fq(&v[576..672]);
                        let c020 = deserialize_fq(&v[672..768]);
                        let c011 = deserialize_fq(&v[768..864]);
                        let c010 = deserialize_fq(&v[864..960]);
                        let c001 = deserialize_fq(&v[960..1056]);
                        let c000 = deserialize_fq(&v[1056..1152]);

                        Ok(Fq12 {
                            c0: Fq6 {
                                c0: Fq2 { c0: c000, c1: c001 },
                                c1: Fq2 { c0: c010, c1: c011 },
                                c2: Fq2 { c0: c020, c1: c021 },
                            },
                            c1: Fq6 {
                                c0: Fq2 { c0: c100, c1: c101 },
                                c1: Fq2 { c0: c110, c1: c111 },
                                c2: Fq2 { c0: c120, c1: c121 },
                            },
                        })
                    }
                    false => return Err(serde::de::Error::custom("invalid length: expected 1152 bytes")),
                }
            }
        }
        d.deserialize_str(Fq12StringVisitor {})
    }
}

fn deserialize_fq(v: &str) -> Fq {
    let mut bytes: Vec<u64> = Vec::new();
    let str_tmp = [
        hex::decode(&v[80..96]).to_owned(),
        hex::decode(&v[64..80]).to_owned(),
        hex::decode(&v[48..64]).to_owned(),
        hex::decode(&v[32..48]).to_owned(),
        hex::decode(&v[16..32]).to_owned(),
        hex::decode(&v[0..16]).to_owned(),
    ];
    for bb in str_tmp.iter() {
        if bb.is_ok() {
            let c = bb.as_ref().unwrap().clone();
            let mut c_array: [u8; 8] = [0; 8];
            c_array.clone_from_slice(c.as_slice());
            bytes.push(u64::from_be_bytes(c_array));
        }
    }

    let mut bytes_array: [u64; 6] = [0; 6];
    bytes_array.clone_from_slice(bytes.as_slice());


    Fq::from_repr(FqRepr(bytes_array)).unwrap()
}

#[cfg(test)]
mod tests {
    extern crate serde_json;

    use super::*;

    use std::fmt::Debug;

    use rand::{Rng, SeedableRng};
    use rand_xorshift::XorShiftRng;

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
