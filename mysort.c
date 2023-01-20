#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

// ベンチマークにより最適な値を決定
#define INSERTION_SORT_THRESHOLD 55

// 論文で推奨された値を指定
#define PARTITION_BLOCK 128

#define MIN(i, j) (i > j ? j : i)
#define SWAP(array, i, j)        \
    {                            \
        int tmp = (array)[i];    \
        (array)[i] = (array)[j]; \
        (array)[j] = tmp;        \
    }

#define DEBUG(...) printf(__VA_ARGS__);
#define DEBUG_ARRAY(array, len)                                 \
    {                                                           \
        printf("[");                                            \
        for (int i = 0; i < len; i++) printf("%d, ", array[i]); \
        printf("]\n");                                          \
    }

void insertion_sort(int *data, int n) {
    for (int i = 1; i < n; i++) {
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

// https://xoshiro.di.unimi.it/xoshiro256plusplus.c
static uint64_t rand_state[4] = {798574385, 328473291321, 459843759845,
                                 48327498239434};
static inline uint64_t rotl(const uint64_t x, int k) {
    return (x << k) | (x >> (64 - k));
}

uint64_t next_rand(void) {
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

// Block Quicksort Partition, hoare finish
// https://drops.dagstuhl.de/opus/volltexte/2016/6389/pdf/LIPIcs-ESA-2016-38.pdf
int block_partition(int *data, int data_len, int pivot) {
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
    }

    if (l_len > 0 && r_len == 0) {
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

// https://en.wikipedia.org/wiki/Quicksort
void quicksort(int *data, int len) {
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

    // ピボットを選択する
    // ピボットを固定値にすると、無限再帰が起こる可能性があるので、
    // 高速なランダム関数を用いてランダムにピボットを決定する。
    int pivot_index = next_rand() % len;
    int pivot = data[pivot_index];

    // パーティション
    int partition = block_partition(data, len, pivot);

    // 短い方だけに再帰するほうが使うスペースが節約できるが
    // 速度に影響はないので行わない。
    quicksort(data, partition);
    quicksort(data + partition, len - partition);
}

void mysort(int *s, int n) {
    int *data = s;
    int len = n;

    quicksort(data, len);
}
