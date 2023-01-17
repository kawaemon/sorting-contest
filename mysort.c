#include <stdio.h>

#define SWAP(array, i, j)        \
    {                            \
        int tmp = (array)[i];    \
        (array)[i] = (array)[j]; \
        (array)[j] = tmp;        \
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

void mysort(int *s, int n) {
    int *data = s;
    int len = n;

    // ソートの意味なし
    if (len < 1) return;

    insertion_sort(data, len);
}
