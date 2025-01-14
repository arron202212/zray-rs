#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_braces)]
//::secrets
// use typing::Tuple, List, Any
use crate::crypto::ecdh_base::EcdhBase;
use crate::interface::{
    ZkayBlockchainInterface, ZkayCryptoInterface, ZkayHomomorphicCryptoInterface,
    ZkayKeystoreInterface, ZkayProverInterface,
};
use crate::types::{
    AddressValue, CipherValue, DataType, KeyPair, PrivateKeyValue, PublicKeyValue, Value,
};
use ark_ff::BigInteger256;
use ark_std::rand;
use jsnark_interface::jsnark_interface::CIRCUIT_BUILDER_JAR;
use rand::RngCore;
use rccell::{RcCell, WeakCell};
use rustc_serialize::hex::ToHex;
use zkay_transaction_crypto_params::params::CryptoParams;
use zkay_utils::run_command::run_command;
fn main() {
    // compile-time length, use `vec![0;len]` for runtime
    let mut bytes = [0; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    // demo-ing both crates, either works
    println!("{}", hex::encode(&bytes));
    println!("{}", bytes.to_hex());
}

use rand::Rng;

fn main1() {
    println!("{}", hex::encode(&rand::thread_rng().gen::<[u8; 16]>()));
}
use std::marker::PhantomData;
#[derive(Clone)]
pub struct EcdhChaskeyCrypto<
    P: ZkayProverInterface + Clone,
    B: ZkayBlockchainInterface<P> + Clone,
    K: ZkayKeystoreInterface<P, B> + Clone,
> {
    key_store: RcCell<K>,
    params: CryptoParams,
    _prover: PhantomData<P>,
    _bc: PhantomData<B>,
}

// class EcdhChaskeyCrypto(EcdhBase):
// params = CryptoParams("ecdh-chaskey")
impl<
        P: ZkayProverInterface + Clone,
        B: ZkayBlockchainInterface<P> + Clone,
        K: ZkayKeystoreInterface<P, B> + Clone,
    > EcdhChaskeyCrypto<P, B, K>
{
    pub fn new(key_store: RcCell<K>) -> Self {
        Self {
            key_store,
            params: CryptoParams::new("ecdh-chaskey".to_owned()),
            _prover: PhantomData,
            _bc: PhantomData,
        }
    }
}
impl<
        P: ZkayProverInterface + Clone,
        B: ZkayBlockchainInterface<P> + Clone,
        K: ZkayKeystoreInterface<P, B> + Clone,
    > EcdhBase<P, B, K> for EcdhChaskeyCrypto<P, B, K>
{
}
impl<
        P: ZkayProverInterface + Clone,
        B: ZkayBlockchainInterface<P> + Clone,
        K: ZkayKeystoreInterface<P, B> + Clone,
    > ZkayCryptoInterface<P, B, K> for EcdhChaskeyCrypto<P, B, K>
{
    fn keystore(&self) -> RcCell<K> {
        self.key_store.clone()
    }

    fn params(&self) -> CryptoParams {
        CryptoParams::new("ecdh-chaskey".to_owned())
    }
    fn _generate_or_load_key_pair(&self, _: &String) -> KeyPair {
        KeyPair::default()
    }

    fn _enc(&self, plain: String, my_sk: String, target_pk: String) -> (Vec<String>, Vec<String>) {
        // # Compute shared key
        let key = Self::_ecdh_sha256(target_pk, my_sk);
        let plain_bytes = plain;

        // # Call java implementation
        let mut iv = hex::encode(&ark_std::rand::thread_rng().gen::<[u8; 16]>());
        let (iv_cipher, _) = run_command(
            vec![
                "java",
                "-Xms4096m",
                "-Xmx16384m",
                "-cp",
                &format!("{CIRCUIT_BUILDER_JAR}"),
                "zkay.ChaskeyLtsCbc",
                "enc",
                &hex::encode(key),
                &hex::encode(iv.clone()),
                &hex::encode(plain_bytes),
            ],
            None,
            false,
        );
        iv.clone().into_bytes().extend(
            i32::from_str_radix(iv_cipher.unwrap().split("\n").last().unwrap(), 16)
                .unwrap()
                .to_be_bytes(),
        );
        let iv_cipher: Vec<u8> = iv.into_bytes(); //.into_iter().flat_map(|v|v.to_string().into_bytes()).collect();

        (
            self.pack_byte_array(iv_cipher, self.params().cipher_chunk_size() as usize),
            vec![],
        )
    }
    fn _dec(&self, mut cipher: Vec<String>, sk: &String) -> (u64, Vec<String>) {
        // # Extract sender address from cipher metadata and request corresponding public key
        let sender_pk = cipher.pop().unwrap();
        // assert!( cipher.len() == self.params.cipher_payload_len);

        // # Compute shared key
        let key = Self::_ecdh_sha256(sender_pk, sk.clone());

        // # Call java implementation
        let iv_cipher = self.unpack_to_byte_array(
            cipher,
            self.params().cipher_chunk_size(),
            self.params().cipher_bytes_payload(),
        );
        let (iv, cipher_bytes) = iv_cipher.split_at(16);
        let (plain, _) = run_command(
            vec![
                "java",
                "-Xms4096m",
                "-Xmx16384m",
                "-cp",
                &format!("{CIRCUIT_BUILDER_JAR}"),
                "zkay.ChaskeyLtsCbc",
                "dec",
                &key.to_hex(),
                &iv.to_hex(),
                &cipher_bytes.to_hex(),
            ],
            None,
            false,
        );
        let plain = u64::from_str_radix(plain.unwrap().split("\n").last().unwrap(), 16).unwrap();

        (plain, vec![])
    }
}

impl<
        P: ZkayProverInterface + Clone,
        B: ZkayBlockchainInterface<P> + Clone,
        K: ZkayKeystoreInterface<P, B> + Clone,
    > ZkayHomomorphicCryptoInterface<P, B, K> for EcdhChaskeyCrypto<P, B, K>
{
    fn do_op(&self, _op: &str, _public_key: Vec<String>, _args: Vec<DataType>) -> Vec<String> {
        vec![]
    }
    fn do_rerand(
        &self,
        _arg: Value<String, CipherValue>,
        _public_key: Vec<String>,
    ) -> (Vec<String>, Vec<u8>) {
        (vec![], vec![])
    }
}
