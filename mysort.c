#include <math.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MIN(i, j) (i > j ? j : i)
#define MAX(i, j) (i > j ? i : j)
#define SWAP(array, i, j)        \
    {                            \
        int tmp = (array)[i];    \
        (array)[i] = (array)[j]; \
        (array)[j] = tmp;        \
    }

void insertion_sort(int *data, int len) {
    for (int i = 1; i < len; i++) {
        if (data[i - 1] > data[i]) {
            // data[i] を適切な位置になるまで左にスライド
            // スワップは遅いので回避する
            // 1 5 7 2 5 0
            //       ↑ slide_from
            // 1 5 7 7 5 0
            //     ↑ slide_from
            // 1 5 5 7 5 0
            // 保存しておいたスライド中の値を本来あるべき場所に置く
            // 1 2 5 7 5 0
            int slide_from = i;
            int sliding_value = data[slide_from];
            do {
                data[slide_from] = data[slide_from - 1];
                slide_from -= 1;
            } while (slide_from > 0 && data[slide_from - 1] > sliding_value);
            data[slide_from] = sliding_value;
        }
    }
}

static inline void sift_down(int *data, int len, int node) {
    while (true) {
        int child = 2 * node + 1;
        if (child >= len) {
            break;
        }
        if (child + 1 < len && data[child] < data[child + 1]) {
            child += 1;
        }
        if (data[node] >= data[child]) {
            break;
        }
        SWAP(data, node, child)
        node = child;
    }
}

void heapsort(int *data, int len) {
    for (int i = (len / 2) - 1; i >= 0; i--) {
        sift_down(data, len, i);
    }
    for (int i = len - 1; i >= 1; i--) {
        SWAP(data, 0, i);
        sift_down(data, i, 0);
    }
}

// https://xoshiro.di.unimi.it/xoshiro256plusplus.c
static uint64_t rand_state[4] = {798574385, 328473291321, 459843759845,
                                 48327498239434};
static inline uint64_t rotl(const uint64_t x, int k) {
    return (x << k) | (x >> (64 - k));
}

static inline uint64_t next_rand(void) {
    const uint64_t result =
        rotl(rand_state[0] + rand_state[3], 23) + rand_state[0];

    const uint64_t t = rand_state[1] << 17;

    rand_state[2] ^= rand_state[0];
    rand_state[3] ^= rand_state[1];
    rand_state[1] ^= rand_state[2];
    rand_state[0] ^= rand_state[3];

    rand_state[2] ^= t;

    rand_state[3] = rotl(rand_state[3], 45);

    return result;
}

// 論文で推奨された値を指定
#define PARTITION_BLOCK 128

// Block Quicksort Partition, hoare finish
// https://drops.dagstuhl.de/opus/volltexte/2016/6389/pdf/LIPIcs-ESA-2016-38.pdf
static inline int block_partition(int *data, int data_len, int pivot) {
    int l_offsets[PARTITION_BLOCK];
    int r_offsets[PARTITION_BLOCK];
    int l_start = 0;
    int r_start = 0;
    int l_len = 0;
    int r_len = 0;
    int l = 0;
    int r = data_len - 1;
    while (r - l + 1 > 2 * PARTITION_BLOCK) {
        // if the left buffer is empty, refill it
        if (l_len == 0) {
            l_start = 0;
            for (int i = 0; i < PARTITION_BLOCK; i++) {
                l_offsets[l_len] = i;
                l_len += (pivot < data[l + i]);
            }
        }

        // do same thing on the right side
        if (r_len == 0) {
            r_start = 0;
            for (int i = 0; i < PARTITION_BLOCK; i++) {
                r_offsets[r_len] = i;
                r_len += (pivot > data[r - i]);
            }
        }

        // rearragement phase
        int num = MIN(l_len, r_len);
        for (int i = 0; i < num; i++) {
            SWAP(data, l + l_offsets[l_start + i], r - r_offsets[r_start + i]);
        }
        l_len -= num;
        r_len -= num;
        l_start += num;
        r_start += num;
        if (l_len == 0) {
            l += PARTITION_BLOCK;
        }
        if (r_len == 0) {
            r -= PARTITION_BLOCK;
        }
    }

    if (r_len > 0 && l_len == 0) {
        while (true) {
            if (pivot < data[l]) {
                SWAP(data, l, r - r_offsets[r_start]);
                r_len -= 1;
                r_start += 1;
                if (r_len == 0) {
                    goto tiny;
                }
            }
            if (l >= r - r_offsets[r_start]) {
                return l + 1;
            }
            l += 1;
        }
    } else if (l_len > 0 && r_len == 0) {
        while (true) {
            if (pivot > data[r]) {
                SWAP(data, r, l + l_offsets[l_start]);
                l_len -= 1;
                l_start += 1;
                if (l_len == 0) {
                    goto tiny;
                }
            }
            if (l + l_offsets[l_start] >= r) {
                return r;
            }
            r -= 1;
        }
    }

tiny:
    while (true) {
        while (data[l] < pivot) {
            l += 1;
        }
        while (data[r] > pivot) {
            r -= 1;
        }
        if (l >= r) {
            break;
        }
        SWAP(data, l, r);
        l += 1;
        r -= 1;
    }
    return l;
}

// ベンチマークにより最適な値を決定
#define INSERTION_SORT_THRESHOLD 55

// https://en.wikipedia.org/wiki/Quicksort
void quicksort(int *data, int len, int recur_limit) {
    // select 7 as pivot
    // 8 4 3 7 6 5 2 1
    // ↑ HI          ↑ LO  swap
    // 1 4 3 7 6 5 2 8
    //    HI ↑     ↑ LO    swap
    // 1 4 3 2 6 5 7 8
    //        LO ↑ ↑ HI    split at the left of HI
    // 1 4 3 2 6 5 | 7 8
    //        LO ↑   ↑ HI

    // ソートの意味なし
    if (len <= 1) {
        return;
    }

    // 十分にソート対象の配列が小さい場合は
    // 小さな配列に対して高速な挿入ソートを用いてソートする。
    // 最適な切り替えタイミングは実測にて特定。
    if (len < INSERTION_SORT_THRESHOLD) {
        insertion_sort(data, len);
        return;
    }

    if (recur_limit == 0) {
        heapsort(data, len);
        return;
    }

    // ピボットを選択する
    // ピボットを固定値にすると、無限再帰が起こる可能性があるので、
    // 高速なランダム関数を用いてランダムにピボットを決定する。
    int pivot_index = next_rand() % len;
    int pivot = data[pivot_index];

    // パーティション
    int partition = block_partition(data, len, pivot);

    // 短い方だけに再帰するほうが使うスペースが節約できるが
    // 速度に影響はないので行わない。
    quicksort(data, partition, recur_limit - 1);
    quicksort(data + partition, len - partition, recur_limit - 1);
}

// NOTE: current implementation does NOT support negative inputs.
void radixsort(int *data, int len) {
    int *temp = malloc(sizeof(int) * len);

    const int int_bits = sizeof(int) * 8;
    int max_bits = 0;
    for (int i = 0; i < len; i++) {
        for (int bit = 0; bit < int_bits; bit++) {
            if ((data[i] & (1 << bit)) != 0) {
                max_bits = MAX(max_bits, bit);
            }
        }
    }

    for (int bit = 0; bit <= max_bits; bit++) {
        int counter = 0;
        for (int i = 0; i < len; i++) {
            temp[i] = data[i];
            if ((data[i] & (1 << bit)) == 0) {
                counter += 1;
            }
        }
        int index[] = {0, counter};
        for (int i = 0; i < len; i++) {
            if ((temp[i] & (1 << bit)) == 0) {
                data[index[0]] = temp[i];
                index[0] += 1;
            } else {
                data[index[1]] = temp[i];
                index[1] += 1;
            }
        }
    }

    free(temp);
}

#define BUCKET_SORT_MAX_NUM 128000  // 512 KiB

void counting_sort(int *data, int len) {
    int count[BUCKET_SORT_MAX_NUM];
    memset(count, 0, sizeof(count));

    for (int i = 0; i < len; i++) {
        count[data[i]] += 1;
    }

    int data_index = 0;
    for (int i = 0; i < BUCKET_SORT_MAX_NUM; i++) {
        for (int j = 0; j < count[i]; j++) {
            data[data_index++] = i;
        }
    }
}

#define INSERTION_SORT_ONLY_THRESHOLD 256

void mysort(int *s, int n) {
    int *data = s;
    int len = n;

    if (len <= 1) {
        return;
    }

    if (len <= INSERTION_SORT_ONLY_THRESHOLD) {
        insertion_sort(data, len);
        return;
    }

    for (int i = 0; i < n; i++) {
        if (data[i] < 0 || data[i] > BUCKET_SORT_MAX_NUM) {
            int recur_limit = ((int)log2(n)) * 2;
            quicksort(data, len, recur_limit);
            return;
        }
    }

    counting_sort(s, n);
}
