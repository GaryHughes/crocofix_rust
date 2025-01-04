#!/usr/bin/python3

import sys
import argparse

# This package is subtree merged
sys.path.append('../../fixorchestra')
from fixorchestra.orchestration import *

from orchestration_fields_generator import *

if __name__ == '__main__':

    parser = argparse.ArgumentParser()
    parser.add_argument('--output', required=True, help='The filename to generate code in')
    parser.add_argument('--module', required=True, help='The name of the generated module')
    parser.add_argument('--orchestration', required=True, help='The orchestration filename to generate code for')

    args = parser.parse_args()

    orchestration = Orchestration(args.orchestration)

    print("generating code in {}".format(args.output))
    print("generating module {}".format(args.module))
    print("from orchestration {}".format(args.orchestration))

    with open(args.output, 'w') as file:
        file.write("#[allow(non_snake_case)]\n")
        file.write("#[allow(dead_code)]\n\n")
        file.write("pub mod {} {{\n\n".format(args.module))

        generate_orchestration_fields(file, orchestration)

        file.write("}\n")

