#define __STDC_FORMAT_MACROS
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>

#define SWAP(array, i, j)        \
    {                            \
        int tmp = (array)[i];    \
        (array)[i] = (array)[j]; \
        (array)[j] = tmp;        \
    }

#ifdef DEBUG
#define DEBUG(...) printf(__VA_ARGS__);
#define SHOW_ARRAY(array, len)                                  \
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

void quicksort(int *data, int len) {
    DEBUG("data = %p, len = %d\n", data, len);
    // select 7 as pivot
    // 8 4 3 7 6 5 2 1
    // ↑ HI          ↑ LO  swap
    // 1 4 3 7 6 5 2 8
    //    HI ↑     ↑ LO    swap
    // 1 4 3 2 6 5 7 8
    //        LO ↑ ↑ HI    split at the left of HI
    // 1 4 3 2 6 5 | 7 8
    //        LO ↑   ↑ HI

    if (len < 2) {
        return;
    }

    // select pivot
    int pivot_index = len / 2;

    // partition
    int hi_index = -1;
    int lo_index = len;
    int pivot = data[pivot_index];
    DEBUG_ARRAY(data, len);
    DEBUG("pivot: %d\n", pivot);
    while (true) {
        DEBUG("begin: hi = %d, low = %d\n", hi_index, lo_index);
        while (hi_index++ < len && data[hi_index] < pivot)
            ;
        while (lo_index-- >= 0 && data[lo_index] > pivot)
            ;
        DEBUG("end: hi = %d, low = %d\n", hi_index, lo_index);
        if (lo_index <= hi_index) {
            break;
        }
        SWAP(data, hi_index, lo_index);
    }

    DEBUG("len = %d, hi = %d, low = %d, part = %d", len, hi_index, lo_index);
    quicksort(data, hi_index);
    quicksort(data + hi_index, len - hi_index);
}

void mysort(int *s, int n) {
    int *data = s;
    int len = n;

    // ソートの意味なし
    if (len < 1) return;

    // insertion_sort(data, len);
    quicksort(data, len);
}
