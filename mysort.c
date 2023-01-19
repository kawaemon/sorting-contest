#include <stdbool.h>
#include <stdio.h>

// ベンチマークにより最適な値を決定。
#define INSERTION_SORT_THRESHOLD 50

#define SWAP(array, i, j)        \
    {                            \
        int tmp = (array)[i];    \
        (array)[i] = (array)[j]; \
        (array)[j] = tmp;        \
    }

#ifdef DEBUG_ENABLED
#define DEBUG(...) printf(__VA_ARGS__);
#define DEBUG_ARRAY(array, len)                                 \
    {                                                           \
        printf("[");                                            \
        for (int i = 0; i < len; i++) printf("%d, ", array[i]); \
        printf("]\n");                                          \
    }
#else
#define DEBUG(...) ;
#define DEBUG_ARRAY(array, len) ;
#endif

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
    if (len < 2) {
        return;
    }

    // 十分にソート対象の配列が小さい場合は
    // より高速な挿入ソートを用いてソートする。
    // 最適な切り替えタイミングは実測にて特定。
    if (len < INSERTION_SORT_THRESHOLD) {
        insertion_sort(data, len);
        return;
    }

    // select pivot
    int pivot_index = len / 2;

    // partition
    int hi_index = -1;
    int lo_index = len;
    int pivot = data[pivot_index];
    while (true) {
        while (hi_index++ < len && data[hi_index] < pivot)
            ;
        while (lo_index-- >= 0 && data[lo_index] > pivot)
            ;
        if (lo_index <= hi_index) {
            break;
        }
        SWAP(data, hi_index, lo_index);
    }

    // 短い方だけに再帰するほうが使うスペースが節約できるが
    // 速度に影響はないので行わない。
    quicksort(data, hi_index);
    quicksort(data + hi_index, len - hi_index);
}

void mysort(int *s, int n) {
    int *data = s;
    int len = n;
    quicksort(data, len);
}
