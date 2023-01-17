void mysort(int *s, int n) {
    int i, k, tmp;
    for (k = 0; k <= n - 2; k++) {
        for (i = n - 1; i >= k + 1; i--) {
            if (s[i - 1] > s[i]) {
                tmp = s[i];
                s[i] = s[i - 1];
                s[i - 1] = tmp;
            }
        }
    }
}
