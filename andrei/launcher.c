#include <errno.h>
#include <limits.h>
#include <mach-o/dyld.h>
#include <spawn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

extern char **environ;

/* A signed native app entry point retains the application's macOS identity while
 * its renderer runs. A shell-script entry point instead has Bash's host identity. */
int main(int argc, char **argv) {
    (void)argc;
    char executable[PATH_MAX], core[PATH_MAX], data[PATH_MAX];
    uint32_t length = sizeof executable;
    if (_NSGetExecutablePath(executable, &length) != 0) return 1;
    char *slash = strrchr(executable, '/');
    if (!slash) return 1;
    *slash = '\0';
    if (snprintf(core, sizeof core, "%s/FilmCraftCore", executable) >= (int)sizeof core) return 1;
    const char *override = getenv("FILMCRAFT_DATA_DIR");
    if (!override || !*override) {
        const char *home = getenv("HOME");
        if (!home || snprintf(data, sizeof data, "%s/Library/Application Support/FilmCraft Andrei", home) >= (int)sizeof data) return 1;
        if (setenv("FILMCRAFT_DATA_DIR", data, 1) != 0) return 1;
    }
    argv[0] = core;
    pid_t child;
    int error = posix_spawn(&child, core, NULL, NULL, argv, environ);
    if (error) { fprintf(stderr, "FilmCraft launch: %s\n", strerror(error)); return 1; }
    int status;
    while (waitpid(child, &status, 0) < 0) {
        if (errno != EINTR) return 1;
    }
    return WIFEXITED(status) ? WEXITSTATUS(status) : 1;
}
