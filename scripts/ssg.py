import sys
from pathlib import Path
from subprocess import Popen

def process_file(f, vars):
    variables = dict([x for x in vars.items()])
    is_processing_vars = True
    ret = ""
    for (num, line) in enumerate(f):
        line = line[:len(line)-1]
        if is_processing_vars:
            # done processing variables
            if len(line) == 0:
                is_processing_vars = False
                # print("Variables recorded:")
                # for (name, val) in variables.items():
                #     print(f"{name}: {val}")
                continue
            process_variables(line, num, variables)
        else:
            ret += process_line(line, variables)
            ret += "\n"

    return ret

def process_variables(line: str, num: int, variables):
        # comment
        if line.find('#') == 0:
            return
        elems = line.split('=', maxsplit=1)
        if len(elems) < 2:
            print(f"line {num}: expected format name=value, no spaces in between.")
        name = elems[0]
        value = elems[1]
        if name in variables:
            print(f"Variable {name} has already been set!")
            sys.exit(1)
        # include file
        if value.find('#') == 0:
            with open(value[1:]) as f:
                print(f"Copy verbatim file {value[1:]}")
                variables[name] = f.read()
        elif value.find('!') == 0:
            with open(value[1:]) as f:
                print(f"Process file {value[1:]}")
                variables[name] = process_file(f, variables)
        else:
            variables[name] = value

# simply substitute the template with variables.
# escape percentage with \%.
# the string cannot be broken in lines
# returns the string
def process_line(line: str, variables):
    is_in_percent = False
    is_escaping = False
    ret = ""
    var = ""

    for c in line:
        if is_in_percent:
            if c == '%':
                sub = variables[var]
                ret += sub
                is_in_percent = False
                var = ""
                continue
            var += c
            continue
        if is_escaping:        
            if c == '%':
                ret += '%'
            else:
                ret += f"\\{c}"
            is_escaping = False
            continue
        if c == '%':
            is_in_percent = True
            continue
        if c == '\\':
            is_escaping = True
            continue
        ret += c
    
    return ret

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"{sys.argv[0]} [input] [output]", file=sys.stderr)
        sys.exit(1)

    with open(sys.argv[1]) as f:
        vars = {}
        path = Path(sys.argv[2])
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(process_file(f, vars), encoding="utf-8")
        print(f"Data written to {sys.argv[2]}")
