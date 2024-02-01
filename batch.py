import argparse
import os
import json
from typing import TypedDict, Optional
import subprocess
import shutil

IMAGE_COL = "img_col"
POST_IMAGE_COL = "post_img_col"

class TemplateCol(TypedDict):
    name: str
    proof_idx: int
    IMAGE_COL: str

class TemplateInstance(TypedDict):
    name: str
    proof_idx: int
    group_idx: int

class EquivalencePair(TypedDict):
    source: TemplateCol
    target: TemplateCol

class AbsorbPair(TypedDict):
    instance_idx: TemplateInstance
    target: TemplateCol

def batch_name_functor(name: str, idx: int) -> str:
    return "batch_" + name + "_" + str(idx)

def compose_first_dsl(proof_name: str, proof_idx: int, group_idx: int) -> dict:
    # First one, we only care about the proof_name
    expose: TemplateCol = {
        "name": proof_name,
        "proof_idx": 0,
        "column_name": POST_IMAGE_COL,
    }
    sepc = {
        "equivalents": [],
        "expose": [expose],
        "absorb": [],
    }
    return sepc

def compose_middle_dsl(proof_name: str, proof_idx: int, group_idx: int) -> dict:
    batch_name = batch_name_functor(proof_name, proof_idx-1)
    instance: TemplateInstance = {
        "name": batch_name,
        "proof_idx": proof_idx - 1,
        "group_idx": group_idx,
    }
    target: TemplateCol = {
        "name": proof_name,
        "proof_idx": proof_idx,
        "column_name": IMAGE_COL,
    }
    absorb: AbsorbPair = {
        "instance_idx": instance,
        "target": target,
    }
    expose: TemplateCol = {
        "name": proof_name,
        "proof_idx": proof_idx,
        "column_name": POST_IMAGE_COL,
    }
    sepc = {
        "equivalents": [],
        "expose": [expose],
        "absorb": [absorb],
    }
    return sepc

def compose_last_dsl(proof_name: str, proof_idx: int, group_idx: int) -> dict:
    batch_name = batch_name_functor(proof_name, proof_idx-1)
    instance: TemplateInstance = {
        "name": batch_name,
        "proof_idx": proof_idx - 1,
        "group_idx": group_idx,
    }
    target: TemplateCol = {
        "name": proof_name,
        "proof_idx": proof_idx,
        "column_name": IMAGE_COL,
    }
    absorb: AbsorbPair = {
        "instance_idx": instance,
        "target": target,
    }
    sepc = {
        "equivalents": [],
        "expose": [],
        "absorb": [absorb],
    }
    return sepc

def write_json(json_dict: dict, file_name: str) -> None:
    json_str = json.dumps(json_dict)
    with open(file_name, mode='w') as f:
        f.write(json_str)

def read_json(file_name: str) -> Optional[dict]:
    try:
        with open(file_name, mode='r') as f:
            return json.load(f)
    except FileNotFoundError:
        return None

def zkwasm_setup_command(zkwasm: str, param: str, proof_name: str, k: int):
    print("Setup command")
    subprocess_cmd = [zkwasm, '--params', param, proof_name,'setup',   '-k', str(k)]
    result = subprocess.run(subprocess_cmd, capture_output=True, text=True)

    print(subprocess_cmd)
    print(result.stdout)
    if result.returncode != 0:
        print(result.stderr)
        print("Setup failed, early exit")
        exit(1)
    print("Setup success")

def zkwasm_prove_command(zkwasm: str, param: str, proof_name: str, wasm: str, public: str, output: str):
    print("Prove command")
    public_list = public.split()
    subprocess_cmd = [zkwasm, '--params', param, proof_name,
                      'prove', '--wasm', wasm, '--output',
                      output, '--public'] + public_list
    result = subprocess.run(subprocess_cmd, capture_output=True, text=True)

    print(subprocess_cmd)
    print(result.stdout)
    if result.returncode != 0:
        print(result.stderr)
        print("Prove failed, early exit")
        exit(1)
    print("Prove success")


def batcher_verify_command(batcher: str, param: str, output: str, challenge: str, proof_name: str):
    print("Batch verify command")
    subprocess_cmd = [batcher, '--param', param, '--output', output, 'verify',
                      '--challenge', challenge, '--info', output+'/'+proof_name+'.loadinfo.json']

    result = subprocess.run(subprocess_cmd, capture_output=True, text=True)

    print(subprocess_cmd)
    print(result.stdout)
    if result.returncode != 0:
        print(result.stderr)
        print("Batch verify failed, early exit")
        exit(1)
    print("Batch verify success")

def batcher_fold_command(batcher: str, param: str, output: str, k: int, challenge: str, proof_name: str, idx: int):
    print("Batch fold command")
    segment_file = output + '/' + proof_name + '.loadinfo.json'
    batch_file = batch_name_functor(proof_name, idx)
    commit_file = batch_file + ".json"
    subprocess_cmd = None
    # The first one is different than others
    if idx != 0:
        prev_batch_file = output + '/' + batch_name_functor(proof_name, idx - 1) + '.loadinfo.json'
        # Seperate by space
        subprocess_cmd = [batcher, '--param', param, '--output', output,
                          'batch', '-k', str(k), '--challenge', challenge,
                          '--info', segment_file, prev_batch_file, '--name',
                          batch_file, '--commits', commit_file] 
    else:
        # First one, we don't have previous batch file
        subprocess_cmd = [batcher, '--param', param, '--output', output,
                          'batch', '-k', str(k), '--challenge', challenge,
                          '--info', segment_file, '--name',
                          batch_file, '--commits', commit_file]
    
    result = subprocess.run(subprocess_cmd, capture_output=True, text=True)

    print(subprocess_cmd)
    print(result.stdout)
    if result.returncode != 0:
        print(result.stderr)
        print("Batch failed, early exit")
        exit(1)
    print("Batch fold success")

def remove_stale_batch_files(param: str, output: str, proof_name: str, idx: int):
    prev_batch_prefix = batch_name_functor(proof_name, idx - 1)
    param_pattern = param + '/' + prev_batch_prefix 
    output_pattern = output + '/' + prev_batch_prefix 
    commit_file = prev_batch_prefix + ".json"
    print("remove stale files: " + param_pattern + " " + output_pattern + " " + commit_file)
    file_list = [output_pattern + '.0.instance.data', output_pattern + '.0.transcripts.data',
                 output_pattern + '.loadinfo.json', param_pattern + '.circuit.data',
                 param_pattern + '.circuit.data.vkey.data', commit_file]
    for file_name in file_list:
      try:
        os.remove(file_name)
      except Exception as e:
        print(f"Error occurred while removing {file_name}: {e}") 

def remove_folder(folder_path: str):
    if os.path.exists(folder_path) and os.path.isdir(folder_path):
        try:
            shutil.rmtree(folder_path)
            print(f"Folder '{folder_path}' has been removed")
        except Exception as e:
            print(f"Error occurred while removing folder: {e}")
    else:
        print(f"Folder '{folder_path}' does not exist")

def main():
    # Parser
    parser = argparse.ArgumentParser(description='Batcher')
    parser.add_argument('--name', help='Proof Serires Name')
    parser.add_argument('--k', help='Space Row')
    parser.add_argument('--wasm', help='WASM Image File')
    parser.add_argument('--public', help='Public Input List, ex 25:i64')
    args = parser.parse_args()
    name = str(args.name)
    k = int(args.k)
    wasm = str(args.wasm)
    public_inputs = str(args.public)

    # To check if zkWASM is set
    zkWASM = os.environ.get('ZKWASM')
    if zkWASM is None:
        print("env var ZKWASM is not set, `export ZKWASM=/path/to/zkwasm` to set it.")
        exit(1)

    # To check if batcher is set
    batcher = os.environ.get('BATCHER')
    if batcher is None:
        print("env var BATCHER is not set, `export BATCHER=/path/to/batcher` to set it.")
        exit(1)
    
    # Default folder
    params_path = 'params'
    output_path = 'output'
    hash_strategy = 'poseidon'

    # Remove the folders in advance
    remove_folder(params_path)
    remove_folder(output_path)

    # Setup
    zkwasm_setup_command(zkWASM, params_path, name, k)

    # Prove
    zkwasm_prove_command(zkWASM, params_path, name, wasm, public_inputs, output_path)

    # If above succeeds, proof loadinfo shall be generated in output folder
    # Read the loadinfo file
    loadinfo = read_json(output_path + '/' + name + '.loadinfo.json')
    length = len(loadinfo['proofs'])
    print(f"Segment Length: {length}")

    # Iterate through the proof series
    for i in range(length):
        if i == 0:
            batcher_verify_command(batcher, params_path, output_path, hash_strategy, name)
            write_json(compose_first_dsl(name, i, i), batch_name_functor(name, i) + ".json")
            batcher_fold_command(batcher, params_path, output_path, k, hash_strategy, name, i)
            batcher_verify_command(batcher, params_path, output_path, hash_strategy, batch_name_functor(name, i))
        elif i == length - 1:
            write_json(compose_last_dsl(name, i, i + 1), batch_name_functor(name, i) + ".json")
            batcher_fold_command(batcher, params_path, output_path, k, hash_strategy, name, i)
            batcher_verify_command(batcher, params_path, output_path, hash_strategy, batch_name_functor(name, i))
            remove_stale_batch_files(params_path, output_path, name, i)
        else:
            write_json(compose_middle_dsl(name, i, i + 1), batch_name_functor(name, i) + ".json")
            batcher_fold_command(batcher, params_path, output_path, k, hash_strategy, name, i)
            batcher_verify_command(batcher, params_path, output_path, hash_strategy, batch_name_functor(name, i))
            remove_stale_batch_files(params_path, output_path, name, i)

if __name__ == '__main__':
    main()
