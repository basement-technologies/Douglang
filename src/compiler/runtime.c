#include <limits.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <process.h>
#include <windows.h>
#else
#include <unistd.h>
#endif

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
static char dv_face_state_path[1024] = {0};
static int dv_face_started = 0;
static int dv_cleaned_up = 0;

static void dv_runtime_cleanup(void);

static void dv_runtime_error(const char *message) {
    fprintf(stderr, "%s\n", message);
    dv_runtime_cleanup();
    exit(1);
}

static void dv_runtime_error_index(const char *format, long long a, long long b) {
    fprintf(stderr, format, a, b);
    fputc('\n', stderr);
    dv_runtime_cleanup();
    exit(1);
}

static void dv_runtime_error_count(const char *format, unsigned long long count) {
    fprintf(stderr, format, count);
    fputc('\n', stderr);
    dv_runtime_cleanup();
    exit(1);
}

static DougValue dv_make_int(long long v) {
    DougValue x;
    x.kind = DV_INT;
    x.i = v;
    x.d = 0;
    x.s = NULL;
    return x;
}

static DougValue dv_make_double(double v) {
    DougValue x;
    x.kind = DV_DOUBLE;
    x.i = 0;
    x.d = v;
    x.s = NULL;
    return x;
}

static DougValue dv_make_string(const char *v) {
    const char *src = v ? v : "";
    DougValue x;
    x.kind = DV_STRING;
    x.i = 0;
    x.d = 0;
    x.s = malloc(strlen(src) + 1);
    if (!x.s) dv_runtime_error("out of memory");
    strcpy(x.s, src);
    return x;
}

static void dv_free_value(DougValue v) {
    if (v.kind == DV_STRING) free(v.s);
}

static DougValue dv_clone_value(DougValue v) {
    switch (v.kind) {
        case DV_INT: return dv_make_int(v.i);
        case DV_DOUBLE: return dv_make_double(v.d);
        case DV_STRING: return dv_make_string(v.s ? v.s : "");
    }
    return dv_make_int(0);
}

static double dv_string_to_double(const char *s) {
    if (!s) return 0.0;
    char *end = NULL;
    double value = strtod(s, &end);
    if (end == s) return 0.0;
    while (*end == ' ' || *end == '\t' || *end == '\n' || *end == '\r') end++;
    if (*end != '\0') return 0.0;
    return value;
}

static long long dv_string_to_int(const char *s) {
    if (!s) return 0;
    char *end = NULL;
    long long value = strtoll(s, &end, 10);
    if (end == s) return 0;
    while (*end == ' ' || *end == '\t' || *end == '\n' || *end == '\r') end++;
    if (*end != '\0') return 0;
    return value;
}

static double dv_as_double(DougValue v) {
    switch (v.kind) {
        case DV_INT: return (double)v.i;
        case DV_DOUBLE: return v.d;
        case DV_STRING: return dv_string_to_double(v.s);
    }
    return 0;
}

static char *dv_to_string(DougValue v) {
    char buf[128];
    switch (v.kind) {
        case DV_STRING: {
            const char *src = v.s ? v.s : "";
            char *r = malloc(strlen(src) + 1);
            if (!r) dv_runtime_error("out of memory");
            strcpy(r, src);
            return r;
        }
        case DV_INT:
            snprintf(buf, sizeof(buf), "%lld", v.i);
            break;
        case DV_DOUBLE: {
            double intpart;
            if (isfinite(v.d) && modf(v.d, &intpart) == 0.0) snprintf(buf, sizeof(buf), "%.1f", v.d);
            else snprintf(buf, sizeof(buf), "%.17g", v.d);
            break;
        }
    }
    char *r = malloc(strlen(buf) + 1);
    if (!r) dv_runtime_error("out of memory");
    strcpy(r, buf);
    return r;
}

static const char *dv_as_cstr(DougValue v) {
    if (v.kind == DV_STRING) return v.s ? v.s : "";
    return dv_to_string(v);
}

static int dv_add_overflows(long long a, long long b, long long *out) {
#if defined(__GNUC__) || defined(__clang__)
    return __builtin_add_overflow(a, b, out);
#else
    if ((b > 0 && a > LLONG_MAX - b) || (b < 0 && a < LLONG_MIN - b)) return 1;
    *out = a + b;
    return 0;
#endif
}

static int dv_sub_overflows(long long a, long long b, long long *out) {
#if defined(__GNUC__) || defined(__clang__)
    return __builtin_sub_overflow(a, b, out);
#else
    if ((b < 0 && a > LLONG_MAX + b) || (b > 0 && a < LLONG_MIN + b)) return 1;
    *out = a - b;
    return 0;
#endif
}

static int dv_mul_overflows(long long a, long long b, long long *out) {
#if defined(__GNUC__) || defined(__clang__)
    return __builtin_mul_overflow(a, b, out);
#else
    double product = (double)a * (double)b;
    if (product > (double)LLONG_MAX || product < (double)LLONG_MIN) return 1;
    *out = a * b;
    return 0;
#endif
}

static void dv_ensure(DougValue **arr, long long *len, long long idx) {
    if (idx >= *len) {
        long long new_len = idx + 1;
        DougValue *new_arr = realloc(*arr, sizeof(DougValue) * new_len);
        if (!new_arr) dv_runtime_error("out of memory");
        *arr = new_arr;
        for (long long k = *len; k < new_len; k++) (*arr)[k] = dv_make_int(0);
        *len = new_len;
    }
}

static DougValue dv_get(long long i) {
    if (i < 0) {
        long long idx = -i - 1;
        if (idx >= dv_left_len) return dv_make_int(0);
        return dv_clone_value(dv_left[idx]);
    }
    if (i >= dv_right_len) return dv_make_int(0);
    return dv_clone_value(dv_right[i]);
}

static void dv_set(long long i, DougValue val) {
    if (i < 0) {
        long long idx = -i - 1;
        if (idx > dv_left_len) {
            dv_runtime_error_index(DV_ERR_INVALID_TAPE_WRITE_FORMAT, i, -dv_left_len - 1);
        }
        dv_ensure(&dv_left, &dv_left_len, idx);
        dv_free_value(dv_left[idx]);
        dv_left[idx] = val;
        return;
    }
    if (i > dv_right_len) {
        dv_runtime_error_index(DV_ERR_INVALID_TAPE_WRITE_FORMAT, i, dv_right_len);
    }
    dv_ensure(&dv_right, &dv_right_len, i);
    dv_free_value(dv_right[i]);
    dv_right[i] = val;
}

static long long dv_shift_for_doug(unsigned long long count) {
    if (count == 0 || count > DV_DOUG_MAX_SAFE_CHAIN_COUNT) {
        dv_runtime_error_count(DV_ERR_DOUG_INDEX_OVERFLOW_FORMAT, count);
    }
    return 1LL << (count - 1);
}

static long long dv_doug_index(long long start, const unsigned long long *counts, size_t len) {
    long long result = start;
    for (size_t i = 0; i < len; i++) {
        long long value = dv_shift_for_doug(counts[i]);
        long long out;
        int overflow = (i % 2 == 0) ? dv_add_overflows(result, value, &out) : dv_sub_overflows(result, value, &out);
        if (overflow) {
            dv_runtime_error_count(DV_ERR_DOUG_INDEX_OVERFLOW_FORMAT, counts[i]);
        }
        result = out;
    }
    return result;
}

static DougValue dv_add(DougValue a, DougValue b) {
    if (a.kind == DV_STRING || b.kind == DV_STRING) {
        char *sa = dv_to_string(a);
        char *sb = dv_to_string(b);
        char *r = malloc(strlen(sa) + strlen(sb) + 1);
        if (!r) dv_runtime_error("out of memory");
        strcpy(r, sa);
        strcat(r, sb);
        DougValue x = dv_make_string(r);
        free(sa);
        free(sb);
        free(r);
        return x;
    }
    if (a.kind == DV_DOUBLE || b.kind == DV_DOUBLE) return dv_make_double(dv_as_double(a) + dv_as_double(b));
    long long out;
    if (dv_add_overflows(a.i, b.i, &out)) return dv_make_double(dv_as_double(a) + dv_as_double(b));
    return dv_make_int(out);
}

static DougValue dv_sub(DougValue a, DougValue b) {
    if (a.kind == DV_DOUBLE || b.kind == DV_DOUBLE) return dv_make_double(dv_as_double(a) - dv_as_double(b));
    if (a.kind == DV_INT && b.kind == DV_INT) {
        long long out;
        if (dv_sub_overflows(a.i, b.i, &out)) return dv_make_double(dv_as_double(a) - dv_as_double(b));
        return dv_make_int(out);
    }
    return dv_make_double(dv_as_double(a) - dv_as_double(b));
}

static DougValue dv_mul(DougValue a, DougValue b) {
    if (a.kind == DV_DOUBLE || b.kind == DV_DOUBLE) return dv_make_double(dv_as_double(a) * dv_as_double(b));
    if (a.kind == DV_INT && b.kind == DV_INT) {
        long long out;
        if (dv_mul_overflows(a.i, b.i, &out)) return dv_make_double(dv_as_double(a) * dv_as_double(b));
        return dv_make_int(out);
    }
    return dv_make_double(dv_as_double(a) * dv_as_double(b));
}

static DougValue dv_div(DougValue a, DougValue b) {
    double y = dv_as_double(b);
    if (y == 0.0) dv_runtime_error(DV_ERR_DIVISION_BY_ZERO);
    return dv_make_double(dv_as_double(a) / y);
}

static DougValue dv_mod(DougValue a, DougValue b) {
    if (a.kind == DV_INT && b.kind == DV_INT) {
        if (b.i == 0) dv_runtime_error(DV_ERR_MODULO_BY_ZERO);
        if (a.i == LLONG_MIN && b.i == -1) dv_runtime_error(DV_ERR_INTEGER_OVERFLOW_MODULO);
        return dv_make_int(a.i % b.i);
    }
    double y = dv_as_double(b);
    if (y == 0.0) dv_runtime_error(DV_ERR_MODULO_BY_ZERO);
    return dv_make_double(fmod(dv_as_double(a), y));
}

static void dv_write_temp_text(const char *path, const char *text) {
    FILE *f = fopen(path, "wb");
    if (!f) return;
    fwrite(text, 1, strlen(text), f);
    fclose(f);
}

static char *dv_escape_state_text(const char *text) {
    size_t len = 1;
    for (const char *p = text; *p; p++) len += (*p == '\\' || *p == '\n' || *p == '|') ? 2 : 1;
    char *out = malloc(len);
    if (!out) dv_runtime_error("out of memory");
    char *w = out;
    for (const char *p = text; *p; p++) {
        if (*p == '\\') {
            *w++ = '\\';
            *w++ = '\\';
        } else if (*p == '\n') {
            *w++ = '\\';
            *w++ = 'n';
        } else if (*p == '|') {
            *w++ = '\\';
            *w++ = 'p';
        } else {
            *w++ = *p;
        }
    }
    *w = '\0';
    return out;
}

static void dv_write_face_state(const char *text, double amp, int speaking) {
    if (!dv_face_state_path[0]) return;
    char *escaped = dv_escape_state_text(text);
    size_t len = strlen(escaped) + 80;
    char *state = malloc(len);
    if (!state) dv_runtime_error("out of memory");
    snprintf(state, len, "%s|%s\n%s|%.17g\n%s|%d\n", DV_TTS_STATE_TEXT_KEY, escaped, DV_TTS_STATE_AMP_KEY, amp, DV_TTS_STATE_SPEAKING_KEY, speaking ? 1 : 0);
    dv_write_temp_text(dv_face_state_path, state);
    free(state);
    free(escaped);
}

static void dv_start_dougterface(void) {
    if (dv_face_started) return;
    dv_face_started = 1;
#ifdef DOUGLANG_HELPER_PATH
#ifdef _WIN32
    char temp_dir[MAX_PATH];
    if (!GetTempPathA(MAX_PATH, temp_dir)) return;
    if (!GetTempFileNameA(temp_dir, "dfc", 0, dv_face_state_path)) return;
    dv_write_temp_text(dv_face_state_path, "");
    _spawnl(_P_NOWAIT, DOUGLANG_HELPER_PATH, "douglang", "--dougterface-helper", dv_face_state_path, NULL);
#else
    snprintf(dv_face_state_path, sizeof(dv_face_state_path), "/tmp/douglang_face_%ld.txt", (long)getpid());
    dv_write_temp_text(dv_face_state_path, "");
    char command[4096];
    snprintf(command, sizeof(command), "'%s' --dougterface-helper '%s' >/dev/null 2>&1 &", DOUGLANG_HELPER_PATH, dv_face_state_path);
    system(command);
#endif
#endif
}

static void dv_show_dougterface(const char *text, int overlap) {
    (void)overlap;
    dv_start_dougterface();
    dv_write_face_state(text, DV_TTS_IDLE_AMP, 0);
}

static void dv_shutdown_dougterface(void) {
    if (!dv_face_state_path[0]) return;
    dv_write_temp_text(dv_face_state_path, DV_TTS_STATE_DONE_MARKER);
}

static void dv_spawn_tts_helper(const char *text, int overlap) {
#ifdef DOUGLANG_HELPER_PATH
#ifdef _WIN32
    char temp_path[MAX_PATH];
    if (!GetTempPathA(MAX_PATH, temp_path)) return;
    if (!GetTempFileNameA(temp_path, "doug", 0, temp_path)) return;
    dv_write_temp_text(temp_path, text);
    if (overlap) {
        _spawnl(_P_NOWAIT, DOUGLANG_HELPER_PATH, "douglang", "--tts-helper-quiet", "overlap", temp_path, dv_face_state_path, NULL);
    } else {
        _spawnl(_P_WAIT, DOUGLANG_HELPER_PATH, "douglang", "--tts-helper-quiet", "speak", temp_path, dv_face_state_path, NULL);
        DeleteFileA(temp_path);
    }
#else
    char temp_path[] = "/tmp/douglang_tts_XXXXXX";
    int fd = mkstemp(temp_path);
    if (fd == -1) return;
    FILE *f = fdopen(fd, "wb");
    if (!f) {
        close(fd);
        unlink(temp_path);
        return;
    }
    fwrite(text, 1, strlen(text), f);
    fclose(f);
    char command[4096];
    snprintf(command, sizeof(command), "'%s' --tts-helper-quiet %s '%s' '%s' '%s'", DOUGLANG_HELPER_PATH, overlap ? "overlap" : "speak", temp_path, dv_face_state_path);
    if (overlap) {
        char async_command[4200];
        snprintf(async_command, sizeof(async_command), "(%s; rm -f '%s') &", command, temp_path);
        system(async_command);
    } else {
        char sync_command[4200];
        snprintf(sync_command, sizeof(sync_command), "%s; rm -f '%s'", command, temp_path);
        system(sync_command);
    }
#endif
#else
    (void)overlap;
    (void)text;
#endif
}

static void dv_speak_text(const char *text, int overlap) {
    printf("%s\n", text);
    dv_show_dougterface(text, overlap);
    dv_spawn_tts_helper(text, overlap);
}

static void dv_tts(DougValue v) {
    char *s = dv_to_string(v);
    dv_speak_text(s, 0);
    free(s);
}

static void dv_ttss(DougValue v) {
    char *s = dv_to_string(v);
    dv_speak_text(s, 1);
    free(s);
}

static void dv_runtime_init(void) {
    dv_ensure(&dv_right, &dv_right_len, 0);
    atexit(dv_runtime_cleanup);
}

static void dv_runtime_cleanup(void) {
    if (dv_cleaned_up) return;
    dv_cleaned_up = 1;
    dv_shutdown_dougterface();
    for (long long i = 0; i < dv_left_len; i++) dv_free_value(dv_left[i]);
    for (long long i = 0; i < dv_right_len; i++) dv_free_value(dv_right[i]);
    free(dv_left);
    free(dv_right);
    dv_left = NULL;
    dv_right = NULL;
    dv_left_len = 0;
    dv_right_len = 0;
}

