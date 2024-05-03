import matplotlib.pyplot as plt
import os

OUT_FOLDER='output'

# structure
# 1. sat / unsat
# 2. heuristics
# 3. input sizes, average runtimes
data = {'satisfiable':{}, 'unsatisfiable':{}}

def get_average(filepath):
    print('Extracting from ' + filepath)
    duration = 0.0
    iteration = 0
    with open(filepath) as file:
        for line in file:
            if 'Profiling results' in line:
                duration += (float)(line.split()[-1][:-2]) / 1000000
                iteration += 1
    return duration / iteration
    


for heuristics in os.listdir(OUT_FOLDER):
    path1 = '/'.join([OUT_FOLDER, heuristics])
    for sat_unsat in os.listdir(path1):
            path2 = '/'.join([path1, sat_unsat])
            for input_size in os.listdir(path2):
                if '.nfs' in input_size:
                    continue
                path3 = '/'.join([path2, input_size])
                if heuristics not in data[sat_unsat]:
                    data[sat_unsat][heuristics] = []
                num_var = int(input_size.split('-')[0])
                if num_var <= 150:
                    data[sat_unsat][heuristics].append((num_var, get_average(path3)))

fig, (ax1, ax2) = plt.subplots(2)
fig.set_figheight(12)
fig.set_figwidth(6)
ax1.set_yscale('log')
ax2.set_yscale('log')
legends = []
markers = ['o', '^', 'D']

for sat_unsat, heuristics in data.items():
    for mark, (heuristic, values) in zip(markers, heuristics.items()):
        sorted_values = sorted(values)
        if sat_unsat == 'satisfiable':
            ax1.plot([x[0] for x in sorted_values], [x[1] for x in sorted_values], label=heuristic, marker=mark)
            legends.append(heuristic)
        elif sat_unsat == 'unsatisfiable':
            ax2.plot([x[0] for x in sorted_values], [x[1] for x in sorted_values], label=heuristic, marker=mark)

ax1.legend(bbox_to_anchor=(1, 0.25))
ax1.title.set_text('Satisfiable runtimes')
ax1.set_xlabel('Number of variables')
ax1.set_ylabel('Runtime (seconds)')

ax2.legend(bbox_to_anchor=(1, 0.25))
ax2.title.set_text('Unsatisfiable runtimes')
ax2.set_xlabel('Number of variables')
ax2.set_ylabel('Runtime (s)')
plt.savefig('result.png', bbox_inches='tight')