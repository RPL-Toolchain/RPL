import re
import sys
import toml

def main():
    counter_path = sys.argv[1] if len(sys.argv) > 1 else "/home/rpl/.VSCodeCounter/2025-07-15_11-27-53/results.md"
    input_path = sys.argv[2] if len(sys.argv) > 2 else "/home/rpl/RPL-Pest/clippy.txt"
    crates_input = sys.argv[3] if len(sys.argv) > 3 else "/home/rpl/RPL-Pest/crates/lintcheck/1000.toml"
    
    crates = toml.load(crates_input)
    crates_dict: dict[str, str] = {}
    for crate_name, crate_data in crates["crates"].items():
        crates_dict[crate_name + '-' + crate_data["version"]] = crate_name
    
    code_dict: dict[str, int] = {}
    with open(counter_path) as f:
        # | path | files | code | comment | blank | total |
        regex = re.compile(r'\| ([^/\(\)]+) \| ([0-9,]+) \| ([0-9,]+) \| ([0-9,]+) \| ([0-9,]+) \| ([0-9,]+) \|')
        for l in f.readlines():
            if regex.match(l):
                m = regex.match(l)
                name = m.group(1)
                code = int(m.group(3).replace(',', ''))
                # print(l)
                if name in crates_dict:
                    code_dict[crates_dict[name]] = code

    times_dict: dict[str, list[float]] = {}
    with open(input_path) as f:
        regex = r"Info\(name='([^']+)', time=\[([0-9, .]+)\]\)"
        index = 0
        for line in f:
            if line.startswith('Processing'):
                continue
            match = re.search(regex, line)
            if match:
                crate_name = match.group(1)
                times = [float(t.strip()) for t in match.group(2).split(',')]
                times_dict[crate_name] = times
                # print(f"{index},{crate_name},{sum(times)/len(times)},{','.join(map(str, times))}")
                # index += 1
        print("序号,包名,行数,平均时间")
        for name, times in times_dict.items():
            assert len(times) == 5
            print(f"{index},{name},{code_dict[name]},{sum(times)/len(times)}")
            index += 1

if __name__ == "__main__":
    main()
