//::os
// from typing::List

use crate::compiler::privacy::circuit_generation::circuit_helper::CircuitHelper;
use crate::config::CFG;
use crate::utils::helpers::hash_file;
use crate::utils::run_command::run_command;
use crate::zkay_ast::ast::indent;

//path to jsnark interface jar
const circuit_builder_jar:&str = "JsnarkCircuitBuilder.jar";
circuit_builder_jar_hash = hash_file(circuit_builder_jar).hex()


pub fn compile_circuit(circuit_dir: &str, javacode: &str)
    // """
    // Compile the given circuit java code and then compile the circuit which it describes using jsnark.

    // :param circuit_dir: output directory
    // :param javacode: circuit code (java class which uses the custom jsnark wrapper API)
    // :raise SubprocessError: if compilation fails
    // """
    {let class_name = cfg.jsnark_circuit_classname;
    let jfile = os.path.join(circuit_dir, class_name + ".java");
    with open(jfile, "w") as f
        f.write(javacode)

    compile_and_run_with_circuit_builder(circuit_dir, class_name, jfile, ["compile"]);}


pub fn compile_and_run_with_circuit_builder(working_dir, class_name, java_file_name, args: Vec<&str>)
    //Compile the circuit java file
   { run_command(["javac", "-cp", f"{circuit_builder_jar}", java_file_name], cwd=working_dir);
    //Run jsnark to generate the circuit
    return run_command(
        ["java", "-Xms4096m", "-Xmx16384m", "-cp", f"{circuit_builder_jar}:{working_dir}", class_name, *args],
        cwd=working_dir, allow_verbose=True)}


pub fn prepare_proof(circuit_dir: &str, output_dir: &str, serialized_args: Vec<i32>)
    // """
    // Generate a libsnark circuit input file by evaluating the circuit in jsnark using the provided input values.

    // :param circuit_dir: directory where the compiled circuit is located
    // :param output_dir: directory, where to store the jsnark output files
    // :param serialized_args: public inputs, public outputs and private inputs in the order in which they are defined in the circuit
    // :raise SubprocessError: if circuit evaluation fails
    // """
    {let serialized_arg_str = [format(arg, "x") for arg in serialized_args];

    //Run jsnark to evaluate the circuit and compute prover inputs
    run_command(["java", "-Xms4096m", "-Xmx16384m", "-cp", f"{circuit_builder_jar}:{circuit_dir}", cfg.jsnark_circuit_classname, "prove", *serialized_arg_str], cwd=output_dir, allow_verbose=True)
}


// """Java circuit code template"""


pub fn get_jsnark_circuit_class_str(circuit: CircuitHelper, crypto_init_stmts: Vec<String>,
                                 fdefs: Vec<String>, circuit_statements: Vec<String>) -> String
    // """
    // Inject circuit and input code into jsnark-wrapper skeleton.

    // :param circuit: the abstract circuit to which this java code corresponds
    // :param fdefs: java code that calls addCryptoBackend for each used crypto backend
    // :param fdefs: java function definition with circuit code for each transitively called function (public calls in this circuit"s function)
    // :param circuit_statements: the java code corresponding to this circuit
    // :return: complete java file as string
    // """
   { let mut function_definitions = fdefs.join("\n\n");
    if function_definitions.is_empty()
        {function_definitions = format!("\n{function_definitions}\n");}

    return format!(r#"
import zkay.ZkayCircuitBase;
import zkay.HomomorphicInput;
import static zkay.ZkayType.ZkUint;
import static zkay.ZkayType.ZkInt;
import static zkay.ZkayType.ZkBool;

public class {circuit_class_name} extends ZkayCircuitBase {{
    public {circuit_class_name}() {{
        super("{circuit_name}", {pub_in_size}, {pub_out_size}, {priv_in_size}, {use_input_hashing});
{crypto_init_stmts}
    }}
{fdefs}
    @Override
    protected void buildCircuit() {{
        super.buildCircuit();
{circuit_statements}
    }}

    public static void main(String[] args) {{
        {circuit_class_name} circuit = new {circuit_class_name}();
        circuit.run(args);
    }}
}}
"#,circuit_class_name=cfg.jsnark_circuit_classname,
                                      circuit_name=circuit.get_verification_contract_name(),
                                      crypto_init_stmts=indent(indent("\n".join(crypto_init_stmts))),
                                      pub_in_size=circuit.in_size_trans,
                                      pub_out_size=circuit.out_size_trans,
                                      priv_in_size=circuit.priv_in_size_trans,
                                      use_input_hashing=&str(cfg.should_use_hash(circuit)).lower(),
                                      fdefs=indent(function_definitions),
                                      circuit_statements=indent(indent("\n".join(circuit_statements))))}
