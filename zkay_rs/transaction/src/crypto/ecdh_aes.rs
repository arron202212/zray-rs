// use typing::Tuple, List, Any

// use Crypto.Cipher::AES

// use zkay::transaction::crypto::params::CryptoParams;
// use zkay::transaction::crypto::ecdh_base::EcdhBase;


// class EcdhAesCrypto(EcdhBase):
//     params = CryptoParams('ecdh-aes')

//     def _enc(self, plain: int, my_sk: int, target_pk: int) -> Tuple[List[int], None]:
//         key = self._ecdh_sha256(target_pk, my_sk)
//         plain_bytes = plain.to_bytes(32, byteorder='big')

//         # Encrypt and extract iv
//         cipher = AES.new(key, AES.MODE_CBC)
//         cipher_bytes = cipher.encrypt(plain_bytes)
//         iv = cipher.iv

//         # Pack iv and cipher
//         iv_cipher = b''.join([iv, cipher_bytes])

//         return self.pack_byte_array(iv_cipher, self.params.cipher_chunk_size), None

//     def _dec(self, cipher: Tuple[int, ...], my_sk: Any) -> Tuple[int, None]:
//         # Extract sender address from cipher metadata and request corresponding public key
//         sender_pk = cipher[-1]
//         cipher = cipher[:-1]
//         assert len(cipher) == self.params.cipher_payload_len

//         # Compute shared key
//         key = self._ecdh_sha256(sender_pk, my_sk)

//         # Unpack iv and cipher
//         iv_cipher = self.unpack_to_byte_array(cipher, self.params.cipher_chunk_size, self.params.cipher_bytes_payload)
//         iv, cipher_bytes = iv_cipher[:16], iv_cipher[16:]

//         # Decrypt
//         cipher = AES.new(key, AES.MODE_CBC, iv=iv)
//         plain_bytes = cipher.decrypt(cipher_bytes)

//         plain = int.from_bytes(plain_bytes, byteorder='big')

//         return plain, None
