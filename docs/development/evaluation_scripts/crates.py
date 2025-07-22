import dataclasses
import re
import os
import sys
import toml
import glob

@dataclasses.dataclass
class Info:
    name: str
    files: int
    code: int
    comment: int
    blank: int
    total: int

def main():
    input = sys.argv[1] if len(sys.argv) > 1 else "/home/rpl/.VSCodeCounter/2025-07-15_11-27-53/results.md"
    crates_input = sys.argv[2] if len(sys.argv) > 2 else "/home/rpl/RPL-Pest/crates/lintcheck/1000.toml"
    check_outputs = sys.argv[3] if len(sys.argv) > 3 else "/home/rpl/RPL-Pest/lintcheck-logs/1000_logs.clippy.*.txt"
    info_list: list[Info] = []
    crates = toml.load(crates_input)
    crates_dict: dict[str, str] = {}
    for crate_name, crate_data in crates["crates"].items():
        crates_dict[crate_name + '-' + crate_data["version"]] = crate_name
    with open(input) as f:
        # | path | files | code | comment | blank | total |
        regex = re.compile(r'\| ([^/\(\)]+) \| ([0-9,]+) \| ([0-9,]+) \| ([0-9,]+) \| ([0-9,]+) \| ([0-9,]+) \|')
        for l in f.readlines():
            if regex.match(l):
                m = regex.match(l)
                info = Info(
                    m.group(1),
                    int(m.group(2).replace(',', '')),
                    int(m.group(3).replace(',', '')),
                    int(m.group(4).replace(',', '')),
                    int(m.group(5).replace(',', '')),
                    int(m.group(6).replace(',', '')),
                )
                # print(l)
                if info.name in crates_dict:
                    info_list.append(info)
                else:
                    print(f"Unknown crate: {info.name}")
        info_list.sort(key=lambda x: x.code)
        print(f"Got {len(info_list)} items")
        # for info in info_list[-50:]:
        #     print(info)
        selected = [crates_dict[info.name] for info in info_list if info.name in crates_dict][19::20]
        print(len(selected))
        if len(glob.glob(check_outputs)) == 0:
            selected_dict = {
                "crates": { name: crates["crates"][name] for name in selected }
            }
            print(toml.dumps(selected_dict))
            print(f"Check output file not found: {check_outputs}")
            return

    info_dict: dict[str, Info] = {
        crates_dict[info.name]: info for info in info_list
    }
    times: dict[str, tuple[int, int]] = {}

    for check_output in glob.glob(check_outputs):
        with open(check_output) as f:
            # target/lintcheck/sources/anstyle-wincon-3.0.9/src/lib.rs:1:25 rpl_interface::timing "11057630 ns has been used during `do_match` after checking anstyle_wincon"
            regex = re.compile(r'target/lintcheck/sources/([^: /]+)/([^:]+):(\d+):(\d+) rpl_interface::timing "(.+?) ns has been used during `do_match` after checking (.+)"')
            for line in f:
                match = regex.match(line)
                if match:
                    crate_name = crates_dict[match.group(1)]
                    info = info_list
                    if not match.group(2).endswith('build.rs') and not match.group(2).endswith('main.rs'):
                        times.setdefault(crate_name, (0, 0))
                        occurrence, total_time = times[crate_name]
                        times[crate_name] = (
                            occurrence + 1,
                            total_time + int(match.group(5))
                        )
    total = 0
    print(f"序号,包名,平均时间,代码行数")
    for crate_name, (occurrence, total_time) in times.items():
        total += 1
        print(f"{total},{crate_name},{total_time/occurrence},{info_dict[crate_name].code}")

if __name__ == "__main__":
    main()
