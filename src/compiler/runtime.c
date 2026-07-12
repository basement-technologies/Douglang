#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef enum { DV_INT, DV_DOUBLE, DV_STRING } DVKind;

typedef struct {
    DVKind kind;
    long long i;
    double d;
    char *s;
} DougValue;

static DougValue *dv_left = NULL;
static long long dv_left_len = 0;
static DougValue *dv_right = NULL;
static long long dv_right_len = 0;
static long long dv_index = 0;

static DougValue dv_make_int(long long v) {
    DougValue x; x.kind = DV_INT; x.i = v; x.d = 0; x.s = NULL; return x;
}
static DougValue dv_make_double(double v) {
    DougValue x; x.kind = DV_DOUBLE; x.i = 0; x.d = v; x.s = NULL; return x;
}
static DougValue dv_make_string(const char *v) {
    DougValue x; x.kind = DV_STRING; x.i = 0; x.d = 0;
    x.s = malloc(strlen(v) + 1); strcpy(x.s, v); return x;
}

static double dv_as_double(DougValue v) {
    switch (v.kind) {
        case DV_INT: return (double)v.i;
        case DV_DOUBLE: return v.d;
        case DV_STRING: return atof(v.s ? v.s : "0");
    }
    return 0;
}

static void dv_ensure(DougValue **arr, long long *len, long long idx) {
    if (idx >= *len) {
        long long new_len = idx + 1;
        *arr = realloc(*arr, sizeof(DougValue) * new_len);
        for (long long k = *len; k < new_len; k++) (*arr)[k] = dv_make_int(0);
        *len = new_len;
    }
}

static DougValue dv_get(long long i) {
    if (i < 0) {
        long long idx = -i - 1;
        if (idx >= dv_left_len) return dv_make_int(0);
        return dv_left[idx];
    } else {
        if (i >= dv_right_len) return dv_make_int(0);
        return dv_right[i];
    }
}

static void dv_set(long long i, DougValue val) {
    if (i < 0) {
        long long idx = -i - 1;
        if (idx > dv_left_len) { fprintf(stderr, "You are literally trolling. Do you know how to count?\n"); exit(1); }
        dv_ensure(&dv_left, &dv_left_len, idx);
        dv_left[idx] = val;
    } else {
        if (i > dv_right_len) { fprintf(stderr, "You are literally trolling. Do you know how to count?\n"); exit(1); }
        dv_ensure(&dv_right, &dv_right_len, i);
        dv_right[i] = val;
    }
}

static char *dv_to_string(DougValue v) {
    char buf[64];
    switch (v.kind) {
        case DV_STRING: {
            char *r = malloc(strlen(v.s ? v.s : "") + 1);
            strcpy(r, v.s ? v.s : ""); return r;
        }
        case DV_INT: sprintf(buf, "%lld", v.i); break;
        case DV_DOUBLE: sprintf(buf, "%g", v.d); break;
    }
    char *r = malloc(strlen(buf) + 1); strcpy(r, buf); return r;
}

static void dv_tts(DougValue v) {
    char *s = dv_to_string(v);
    printf("%s\n", s);
    free(s);
}

static const char *dv_as_cstr(DougValue v) {
    if (v.kind == DV_STRING) return v.s ? v.s : "";
    return dv_to_string(v);
}

static long long dv_as_int(DougValue v) {
    switch (v.kind) {
        case DV_INT: return v.i;
        case DV_DOUBLE: return (long long)v.d;
        case DV_STRING: return (long long)atoll(v.s ? v.s : "0");
    }
    return 0;
}

static DougValue dv_add(DougValue a, DougValue b) {
    if (a.kind == DV_STRING || b.kind == DV_STRING) {
        char *sa = dv_to_string(a);
        char *sb = dv_to_string(b);
        char *r = malloc(strlen(sa) + strlen(sb) + 1);
        strcpy(r, sa); strcat(r, sb);
        DougValue x = dv_make_string(r);
        free(sa); free(sb); free(r);
        return x;
    }
    if (a.kind == DV_DOUBLE || b.kind == DV_DOUBLE)
        return dv_make_double(dv_as_double(a) + dv_as_double(b));
    return dv_make_int(a.i + b.i);
}

static DougValue dv_arith(DougValue a, DougValue b, char oper) {
    if (oper == '/' || a.kind == DV_DOUBLE || b.kind == DV_DOUBLE) {
        double x = dv_as_double(a), y = dv_as_double(b);
        switch (oper) {
            case '-': return dv_make_double(x - y);
            case '*': return dv_make_double(x * y);
            case '/': return dv_make_double(x / y);
        }
    }
    switch (oper) {
        case '-': return dv_make_int(a.i - b.i);
        case '*': return dv_make_int(a.i * b.i);
        case '%': return dv_make_int(a.i % b.i);
    }
    return dv_make_int(0);
}
