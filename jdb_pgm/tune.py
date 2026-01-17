#!/usr/bin/env python

import os
import subprocess
import ConfigSpace as CS
import ConfigSpace.hyperparameters as CSH
from dehb import DEHB

CARGO_CMD = [
    "cargo",
    "run",
    "--quiet",
    "--example",
    "score",
    "--release",
    "--",
]

MIN_FIDELITY = 1 
MAX_FIDELITY = 10 
ITERATIONS = 20

def target_function(config, fidelity=None, **kwargs):
    env = os.environ.copy()

    block_len = config["block_len"]
    epsilon = config["epsilon"]
    ex_penalty = config["ex_penalty"]

    # Block Len is compile-time (build.rs)
    env["PC_BLOCK_LEN"] = str(block_len)

    # Epsilon & Penalty are runtime (arg)
    cmd = CARGO_CMD + ["--epsilon", str(epsilon), "--ex-penalty", str(ex_penalty)]

    try:
        # Build & Run
        # Capture output. 
        # Note: If compilation happens, it might take time.
        # Strict separation: `cargo build` first? 
        # `cargo run` handles it.
        result = subprocess.run(
            cmd, env=env, check=True, capture_output=True, text=True
        )

        score_str = result.stdout.strip()
        # Last line needs to be the score if there are debug prints
        lines = score_str.splitlines()
        score = 0.0
        if lines:
            try:
                score = float(lines[0]) # score.rs prints score first, then debug to stderr
            except ValueError:
                 print(f"Parse error: {lines}")
                 return {"fitness": 1e12, "cost": 1.0}
        
        # Maximize Score => Minimize Fitness
        # Fitness = 1e9 / (Score + 1e-6)
        fitness = 1e9 / (score + 1e-6)
        
        print(f"  [Eval] Score: {score:.4f} | B={block_len} Eps={epsilon} Pen={ex_penalty}")
        return {"fitness": fitness, "cost": 1.0}

    except subprocess.CalledProcessError as e:
        print(f"Error: {e}")
        if e.stderr:
            print(f"Stderr: {e.stderr}")
        return {"fitness": 1e12, "cost": 1.0}

def get_config_space():
    cs = CS.ConfigurationSpace()
    
    # Block Length
    block_len = CSH.OrdinalHyperparameter(
        "block_len", [128, 256, 512, 1024], default_value=256
    )
    
    # Epsilon (PGM precision)
    epsilon = CSH.OrdinalHyperparameter(
        "epsilon", [8, 16, 32, 64], default_value=16
    )

    # Exception Penalty (PFOR)
    ex_penalty = CSH.OrdinalHyperparameter(
        "ex_penalty", [1, 2, 4, 8, 16], default_value=1
    )
    
    cs.add(block_len)
    cs.add(epsilon)
    cs.add(ex_penalty)
    return cs

if __name__ == "__main__":
    cs = get_config_space()
    print(f"Starting Optimization...")
    
    dehb = DEHB(
        f=target_function,
        cs=cs,
        dimensions=len(cs),
        min_fidelity=MIN_FIDELITY,
        max_fidelity=MAX_FIDELITY,
        n_workers=1,
        output_path="./dehb_logs",
        log_level="ERROR"
    )

    dehb.run(fevals=ITERATIONS)

    best_config = dehb.vector_to_configspace(dehb.inc_config)
    fitness = dehb.inc_score
    score = (1e9 / fitness) - 1e-6

    print("\n" + "="*40)
    print(f"Best Config Found:")
    print(f"Score: {score:.4f}")
    for k, v in best_config.items():
        print(f"  {k}: {v}")
    print("="*40 + "\n")
