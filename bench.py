import subprocess
import re
import sys

regex = re.compile(r"\[(?P<time>[\d.]*) (?P<unit>\w)s [\d.]* \ws [\d.]+ \ws\]")

print("threshold,small,medium,large")
sys.stdout.flush()

with open("mysort.base.c", "r") as f:
    code = f.read()

for i in range(50, 150, 10):
    print(f"benchmarking {i}", file=sys.stderr)
    sys.stdout.flush()

    with open("mysort.c", "w") as f:
        f.write(f"#define INSERTION_SORT_THRESHOLD {i}\n" + code)

    if subprocess.run(["zsh", "-c" , f"just bench"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode != 0:
        exit()

    with open("./performance", "r") as f:
        perf = f.read()

    times = [str(i)]
    for time, unit in regex.findall(perf):
        time = float(time)
        if unit == "m":
            time *= 1000.0
        times.append(str(time))
    print(",".join(times))
    sys.stdout.flush()
