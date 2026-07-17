#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <process.h>
#include <windows.h>
#else
#include <sys/wait.h>
#include <unistd.h>
#endif

static const char *DOUGLANG_HELPER_PATH = __DOUGLANG_HELPER_PATH__;
static const char *DOUGLANG_SOURCE = __DOUGLANG_SOURCE__;
static const char *DOUGLANG_LINKS[] = {__DOUGLANG_LINKS__};
static const int DOUGLANG_LINK_COUNT = __DOUGLANG_LINK_COUNT__;

static int write_source_file(char *path, size_t path_len) {
#ifdef _WIN32
    char temp_dir[MAX_PATH];
    if (!GetTempPathA(MAX_PATH, temp_dir)) return 1;
    if (!GetTempFileNameA(temp_dir, "dlg", 0, path)) return 1;
#else
    snprintf(path, path_len, "/tmp/douglang_compiled_XXXXXX");
    int fd = mkstemp(path);
    if (fd == -1) return 1;
    FILE *made = fdopen(fd, "wb");
    if (!made) { close(fd); return 1; }
    fwrite(DOUGLANG_SOURCE, 1, strlen(DOUGLANG_SOURCE), made);
    fclose(made);
    return 0;
#endif
    FILE *f = fopen(path, "wb");
    if (!f) return 1;
    fwrite(DOUGLANG_SOURCE, 1, strlen(DOUGLANG_SOURCE), f);
    fclose(f);
    return 0;
}

int main(void) {
    char source_path[4096];
    if (write_source_file(source_path, sizeof(source_path)) != 0) {
        fprintf(stderr, "couldn't create compiled Douglang source file\n");
        return 1;
    }

#ifdef _WIN32
    const int argc = 3 + (DOUGLANG_LINK_COUNT * 2) + 1;
    const char **argv = (const char **)calloc((size_t)argc, sizeof(char *));
    if (!argv) return 1;
    int at = 0;
    argv[at++] = "douglang";
    argv[at++] = "--run-source-helper";
    argv[at++] = source_path;
    for (int i = 0; i < DOUGLANG_LINK_COUNT; i++) {
        argv[at++] = "--link";
        argv[at++] = DOUGLANG_LINKS[i];
    }
    argv[at] = NULL;
    intptr_t result = _spawnv(_P_WAIT, DOUGLANG_HELPER_PATH, argv);
    free(argv);
    DeleteFileA(source_path);
    if (result == -1) {
        fprintf(stderr, "couldn't run douglang runtime helper\n");
        return 1;
    }
    return (int)result;
#else
    pid_t pid = fork();
    if (pid == -1) return 1;
    if (pid == 0) {
        int argc = 3 + (DOUGLANG_LINK_COUNT * 2) + 1;
        char **argv = (char **)calloc((size_t)argc, sizeof(char *));
        if (!argv) _exit(1);
        int at = 0;
        argv[at++] = "douglang";
        argv[at++] = "--run-source-helper";
        argv[at++] = source_path;
        for (int i = 0; i < DOUGLANG_LINK_COUNT; i++) {
            argv[at++] = "--link";
            argv[at++] = (char *)DOUGLANG_LINKS[i];
        }
        argv[at] = NULL;
        execv(DOUGLANG_HELPER_PATH, argv);
        _exit(1);
    }
    int status = 0;
    waitpid(pid, &status, 0);
    unlink(source_path);
    if (WIFEXITED(status)) return WEXITSTATUS(status);
    return 1;
#endif
}
