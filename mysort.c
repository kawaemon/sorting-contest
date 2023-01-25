#include <math.h>
#include <stdbool.h>
#include <string.h>

#define MIN(i, j) (i > j ? j : i)
#define MAX(i, j) (i > j ? i : j)
#define SWAP(array, i, j)        \
    {                            \
        int tmp = (array)[i];    \
        (array)[i] = (array)[j]; \
        (array)[j] = tmp;        \
    }

// https://en.wikipedia.org/wiki/Insertion_sort
// 小さい配列や、すでにほとんどソートされている配列に対して高速
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

// node (index) 以下のノードが heap property を満たすようにノードを入れ替える
static inline void down_heap(int *data, int len, int node) {
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

// https://en.wikipedia.org/wiki/Heapsort
// https://en.wikipedia.org/wiki/Binary_heap
// 2 分ヒープを配列内に配置してソートする。
// Quicksort より平均的に遅いが、最悪でも O(nlogn) で終了する。
// Quicksort は pivot の選び方が悪いと O(n^2) まで悪化する可能性がある。
void heapsort(int *data, int len) {
    for (int i = (len / 2) - 1; i >= 0; i--) {
        down_heap(data, len, i);
    }
    for (int i = len - 1; i >= 1; i--) {
        SWAP(data, 0, i);
        down_heap(data, i, 0);
    }
}

// なるべく CPU のキャッシュに当たりやすいサイズを狙う
#define PARTITION_BLOCK 128

// Block Quicksort Partition
// https://drops.dagstuhl.de/opus/volltexte/2016/6389/pdf/LIPIcs-ESA-2016-38.pdf
static inline int block_partition(int *data, int len, int pivot) {
    int l_offsets[PARTITION_BLOCK];
    int r_offsets[PARTITION_BLOCK];
    int l_start = 0;
    int r_start = 0;
    int l_len = 0;
    int r_len = 0;
    int l = 0;
    int r = len - 1;
    while (r - l + 1 > 2 * PARTITION_BLOCK) {
        if (l_len == 0) {
            l_start = 0;
            for (int i = 0; i < PARTITION_BLOCK; i++) {
                l_offsets[l_len] = i;
                l_len += (pivot < data[l + i]);
            }
        }
        if (r_len == 0) {
            r_start = 0;
            for (int i = 0; i < PARTITION_BLOCK; i++) {
                r_offsets[r_len] = i;
                r_len += (pivot > data[r - i]);
            }
        }
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
// https://en.wikipedia.org/wiki/Introsort
static inline void introsort_internal(int *data, int len, int recur_limit) {
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
    if (len < INSERTION_SORT_THRESHOLD) {
        insertion_sort(data, len);
        return;
    }

    // 再帰が深くなりすぎた (= pivot の選択が悪い場合が何回も続いた）場合は、
    // 必ず O(nlogn) で終了する heapsort を使う。
    if (recur_limit == 0) {
        heapsort(data, len);
        return;
    }

    int pivot_index = len / 2;
    int pivot = data[pivot_index];

    int partition = block_partition(data, len, pivot);

    introsort_internal(data, partition, recur_limit - 1);
    introsort_internal(data + partition, len - partition, recur_limit - 1);
}

void introsort(int *data, int len) {
    introsort_internal(data, len, log2(len) * 2);
}

// 偶数前提で境界を計算しているため、奇数にするならコードの変更が必要
#define BUCKET_SORT_ELEMENT_SIZE 1024  // int が 32bit なら 512 KiB
// elements: 4
// count:   0   1  2  3
//                 ↑ center
// value:  -2  -1  0  1

// これを Bucket Sort を呼べるかどうかは正直わからない。
// (原案は Bucket Sort だが、閃きで実装)
// https://en.wikipedia.org/wiki/Bucket_sort
// 制約: data の全ての要素が次を満たさなければならない
// 0 <= data[i] < BUCKET_SORT_ELEMENT_SIZE
void bucket_sort(int *data, int len) {
    int count[BUCKET_SORT_ELEMENT_SIZE];
    memset(count, 0, sizeof(count));

    for (int i = 0; i < len; i++) {
        count[data[i]] += 1;
    }

    int data_index = 0;
    for (int i = 0; i < BUCKET_SORT_ELEMENT_SIZE; i++) {
        for (int j = 0; j < count[i]; j++) {
            data[data_index++] = i;
        }
    }
}

void mysort(int *s, int n) {
    int *data = s;
    int len = n;

    if (len <= 1) {
        return;
    }

    if (len <= 65) {
        introsort(data, len);
        return;
    }

    for (int i = 0; i < len; i++) {
        if (!(0 <= data[i] && data[i] < BUCKET_SORT_ELEMENT_SIZE)) {
            introsort(data, len);
            return;
        }
    }

    bucket_sort(data, len);
}
