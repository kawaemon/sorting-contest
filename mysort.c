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
            // data[i] を 適切な位置になるまで左にスライド
            // 1 5 7 2 5 0
            //       ↑ slide_from
            int slide_from = i;
            do {
                SWAP(data, slide_from - 1, slide_from);
                slide_from -= 1;
            } while(slide_from > 0 && data[slide_from - 1] > data[slide_from]);
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
