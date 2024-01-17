// import functools
// import os
// from abc import ABCMeta, abstractmethod
// from multiprocessing import Pool, Value
// from typing import List, Tuple
use crate::zkay_ast::ast::ConstructorOrFunctionDefinition;
use crate::compiler::privacy::circuit_generation::circuit_helper::CircuitHelper;
use crate::compiler::privacy::proving_scheme::proving_scheme::{ProvingScheme, VerifyingKeyMeta};
use crate::{zk_print, config::CFG};
use crate::utils::progress_printer::print_step;
use crate::utils::timer::time_measure;
use rayon::prelude::*;
use std::path::Path;
extern crate num_cpus;
use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::fs::File;
lazy_static! {
    pub static ref finish_counter: Mutex<i32> = Mutex::new(0);
    pub static ref c_count: Mutex<i32> = Mutex::new(0);
}
pub trait CircuitGenerator{
}
// class CircuitGeneratorBase(metaclass=ABCMeta)
pub struct CircuitGeneratorBase<T:ProvingScheme,V> {
    circuits: BTreeMap<ConstructorOrFunctionDefinition, CircuitHelper<V>>,
    circuits_to_prove: Vec<CircuitHelper<V>>,
    proving_scheme: T,
    output_dir: String,
    parallel_keygen: bool,
    p_count: i32,
}

impl<T:ProvingScheme,V> CircuitGeneratorBase<T,V> {
    // """
    // A circuit generator takes an abstract circuit representation and turns it into a concrete zk-snark circuit.

    // It also handles prover/verification key generation and parsing, and generates the verification contracts using the supplied
    // proving scheme.
    // """

    pub fn new(
        circuits: Vec<CircuitHelper<V>>,
        proving_scheme: T,
        output_dir: String,
        parallel_keygen: bool,
    )
    // """
    // Create a circuit generator instance

    // :param circuits: list which contains the corresponding circuit helper for every function in the contract which requires verification
    // :param proving_scheme: the proving scheme instance to be used for verification contract generation
    // :param output_dir: base directory where the zkay compilation output is located
    // :param parallel_keygen: if true, multiple python processes are used to generate keys in parallel
    // """
    {
        let circuits_to_prove = circuits
            .iter()
            .filter_map(|c| {
                if c.requires_verification() && c.fct.can_be_external && c.fct.has_side_effects {
                    Some(c)
                } else {
                    None
                }
            })
            .collect();
        let p_count = (circuits_to_prove.len() as i32).min(num_cpus::get());
        Self {
            circuits: circuits
                .iter()
                .map(|circ| (circ.fct.clone(), circ.clone()))
                .collect(),
            circuits_to_prove,
            proving_scheme,
            output_dir,
            parallel_keygen,
            p_count,
        }
    }

    pub fn generate_circuits(&self, import_keys: bool)
    // """
    // Generate circuit code and verification contracts based on the provided circuits and proving scheme.

    // :param import_keys: if false, new verification and prover keys will be generated, otherwise key files for all verifiers
    //                     are expected to be already present in the respective output directories
    // """
    //Generate proof circuit code

    //Compile circuits
    {
        let _c_count = self.circuits_to_prove.len();
        zk_print!("Compiling {c_count} circuits...");

        let gen_circs =
            |circuit: CircuitHelper| -> bool { self._generate_zkcircuit(import_keys, circuit) };
        // with
        time_measure("circuit_compilation", true);
        let modified: Vec<_> = if CFG.lock().unwrap().is_unit_test {
            self.circuits_to_prove.iter().map(gen_circs).collect()
        } else {
            // with Pool(processes=self.p_count) as pool
            self.circuits_to_prove.par_iter().map(gen_circs).collect()
        };

        if import_keys {
            for path in self.get_all_key_paths() {
                if !Path::new(path).try_exists().map_or(false, |v| v) {
                    assert!(false, "Zkay contract import failed: Missing keys");
                }
            }
        } else {
            let modified_circuits_to_prove: Vec<_> = modified
                .iter()
                .zip(&self.circuits_to_prove)
                .filter_map(|(t, circ)| {
                    if t || !self
                        ._get_vk_and_pk_paths(circ)
                        .all(|p| Path::new(p).try_exist().map_or(false, |v| v))
                    {
                        Some(circ)
                    } else {
                        None
                    }
                })
                .collect();
            //Generate keys in parallel
            zk_print!("Generating keys for {c_count} circuits...");
            time_measure("key_generation", true);
            {
                if self.parallel_keygen && !CFG.lock().unwrap().is_unit_test {
                    let counter =0;// Value("i", 0);
                    // with Pool(processes=self.p_count, initializer=self.__init_worker, initargs=(counter, c_count,)) as pool
                    {
                        modified_circuits_to_prove
                            .par_iter()
                            .for_each(self._generate_keys_par);
                    }
                } else {
                    for circ in modified_circuits_to_prove {
                        self._generate_keys(circ);
                    }
                }
            }
        }

        print_step("Write verification contracts");
        {
            for circuit in self.circuits_to_prove {
                let vk = self._parse_verification_key(circuit);
                let pk_hash = self._get_prover_key_hash(circuit);
                let f = File::create(Path::new(
                    self.output_dir.join(circuit.verifier_contract_filename),
                ))
                .expect("");
                {
                    let primary_inputs = self._get_primary_inputs(circuit);
                    f.write_all(
                        self.proving_scheme
                            .generate_verification_contract(vk, circuit, primary_inputs, pk_hash)
                            .as_bytes(),
                    );
                }
            }
        }
    }
    pub fn get_all_key_paths(self) -> Vec<String>
// """Return paths of all key files for this contract."""
    {
        let paths = vec![];
        for circuit in self.circuits_to_prove {
            paths.extend(self._get_vk_and_pk_paths(circuit));
        }
        paths
    }

    pub fn get_verification_contract_filenames(self) -> Vec<String>
// """Return file paths for all verification contracts generated by this CircuitGeneratorBase"""
    {
        self.circuits_to_prove
            .iter()
            .map(|circuit| {
                Path::new(self.output_dir)
                    .join(circuit.verifier_contract_filename)
                    .to_string()
            })
            .collect()
    }

    // @staticmethod
    pub fn __init_worker(counter: i32, total_count: i32) {
        finish_counter.lock().unwrap() = counter;
        c_count.lock().unwrap() = total_count;
    }

    pub fn _generate_keys_par(&self, circuit: CircuitHelper<V>) {
        self._generate_keys(circuit);

        finish_counter.lock().unwrap() += 1;
        zk_print!(
            r#"Generated keys for circuit "\"{}\" [{}/{c_count}]"#,
            circuit.verifier_contract_type.code(),
            finish_counter.value,
        );
    }

    pub fn _get_circuit_output_dir(&self, circuit: CircuitHelper<V>)
    // """Return the output directory for an individual circuit"""
    {
        self.output_dir.join(
            CFG.lock()
                .unwrap()
                .get_circuit_output_dir_name(circuit.get_verification_contract_name()),
        )
    }

    pub fn _get_vk_and_pk_paths(&self, circuit: CircuitHelper<V>) -> Vec<String>
// """Return a tuple which contains the paths to the verification and prover key files."""
    {
        let output_dir = self._get_circuit_output_dir(circuit);
        self.get_vk_and_pk_filenames()
            .iter()
            .map(|fname| output_dir.join(fname).to_string())
            .collect()
    }

    // @abstractmethod
    pub fn _generate_zkcircuit(&self, import_keys: bool, circuit: CircuitHelper<V>) -> bool
// """
        // Generate code and compile a single circuit.

        // When implementing a new backend, this function should generate a concrete circuit representation, which has
        // a) circuit IO corresponding to circuit.sec_idfs/output_idfs/input_idfs
        // b) logic corresponding to the non-CircCall statements in circuit.phi
        // c) a), b) and c) for the circuit associated with the target function for every CircCall statement in circuit.phi

        // The output of this function should be in a state where key generation can be invoked immediately without further transformations
        // (i.e. any intermediary compilation steps should also happen here).

        // It should be stored in self._get_circuit_output_dir(circuit)

        // :return: true if the circuit was modified since last generation (need to generate new keys)
        // """
        // pass
    {
        false
    }

    // @abstractmethod
    pub fn _generate_keys(&self, circuit: CircuitHelper<V>) {}
    // """Generate prover and verification keys for the circuit stored in self._get_circuit_output_dir(circuit)."""
    // pass

    // @classmethod
    // @abstractmethod
    pub fn get_vk_and_pk_filenames() -> Vec<String> {
        vec![]
    }
    // pass

    // @abstractmethod
    pub fn _parse_verification_key<VK>(&self, circuit: CircuitHelper<V>) -> dyn VerifyingKeyMeta<Output=VK>
// """Parse the generated verificaton key file and return a verification key object compatible with self.proving_scheme"""
    {
        self.proving_scheme.VerifyingKey.create_dummy_key()
    }

    // @abstractmethod
    pub fn _get_prover_key_hash(&self, circuit: CircuitHelper<V>) -> Vec<u8> {
        vec![]
    }
    // pass

    pub fn _get_primary_inputs(&self, circuit: CircuitHelper<V>) -> Vec<String>
// """
        // Return list of all public input locations
        // :param circuit: abstract circuit representation
        // :return: list of location strings, a location is either an identifier name or an array index
        // """
    {
        let inputs = circuit.public_arg_arrays.clone();

        if CFG.lock().unwrap().should_use_hash(circuit) {
            vec![self.proving_scheme.hash_var_name.clone()]
        } else {
            let primary_inputs = vec![];
            for (name, count) in inputs {
                primary_inputs.extend((0..count).map(|i| format!("{name}[{i}]")).collect())
            }
            primary_inputs
        }
    }
}
