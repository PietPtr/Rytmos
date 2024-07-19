import random

# subdivs = [
#     '┈┈',
#     '┈┥',
#     '┈━',
#     '┥┈',
#     '┥┥',
#     '┥━',
#     '━┥',
#     '━━'
# ]

from itertools import combinations_with_replacement, product
from pprint import pprint


glyphs = [
    '┈',
    '┥',
    '━',
]

def generate_rhythm_set(length):
    rhythm_set = set()
    for rhythm in product(glyphs, repeat=length):
        rhythm = list(rhythm)
        for i in range(len(rhythm) - 1):
            if rhythm[i] == '━' and rhythm[i+1] == '┈':
                rhythm[i] = '┥'
        
        rhythm_set.add("".join(rhythm))

    return rhythm_set

def generate_valid(length):
    rhythm = [random.choice(glyphs) for _ in range(length)]

    for i in range(len(rhythm) - 1):
        sub = (rhythm[i:i+2])
        if sub == ['━', '┈']:
            rhythm[i] = '┥'


    return "".join(rhythm)

def determine_rhythm_set(length):
    rhythm_set = set()

    i = 0
    last_i = 0
    while True:
        rhythm = generate_valid(length)
        if rhythm not in rhythm_set:
            rhythm_set.add(rhythm)
            # print(f"{len(rhythm_set)} {rhythm}")
            last_i = i

        if i - last_i > 1000:
            break

        i += 1

    return rhythm_set

def model(n, a0, a1):
    if n == 0:
        return a0
    elif n == 1:
        return a1
    else:
        a = [a0, a1]
        for i in range(2, n + 1):
            a.append(3 * a[i - 1] - a[i - 2])
        return a[n]

for i in range(1,40):
    # rset = len(generate_rhythm_set(i))
    rset = model(i + 1, 0, 1)
    print(f"{i} {rset}")

i = 1
for rhythm in sorted(list(generate_rhythm_set(4))):
    print(i, rhythm)
    i += 1

data = sorted(list(generate_rhythm_set(4)))

for i in range(0, len(data), 5):
    print('\t'.join(map(str, data[i:i+5])))

# 2 8
# 3 21
# 4 55
# 5 144
# 6 377
# 7 987
# 8 2584
# 9 6765
# 10 17711
# 11 46368
# 12 121393
# 13 317811
# 14 832040
# 15 2178309
# 16 5702887