import sys

with open('gateway/src/engine/mod.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if 'impl Engine {' in line:
        new_lines.append(line)
        continue
    new_lines.append(line)

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.writelines(new_lines)
