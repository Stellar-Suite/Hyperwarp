// #include <GL/gl.h>
// #include <FTGL/ftgl.h>
#include <stdio.h>
#include <dlfcn.h>
#include <stdlib.h>
#include <unistd.h>
static int DEBUG = 0;

static void init() __attribute__((constructor));
// void performOverlay();

// void (*_glfwSwapBuffers)() = NULL;

/*void glfwSwapBuffers()
{
  if (_glfwSwapBuffers == NULL) {
    void *handle = dlopen("libglfw.so", RTLD_LAZY);
    if (handle) {
      _glfwSwapBuffers = dlsym(handle, "glfwSwapBuffers");
      dlclose(handle);
    }
  }

  performOverlay();

  _glfwSwapBuffers();
}*/

static void init()
{
  if (DEBUG)
    printf("Glue: Init");
}

void prepare()
{
  if (getenv("DEBUG_HW") != NULL)
  {
    DEBUG = 1;
  }
  // printf("debug %s",getenv("DEBUG"));
}

/* void performOverlay()
{
  static FTGLfont *font = NULL;

  if (font == NULL) {
    font = ftglCreateTextureFont("font.ttf");
    ftglSetFontFaceSize(font, 16, 16);
  }

  glColor4f(0, 0, 0, 1);

  glMatrixMode(GL_PROJECTION);
  glPushMatrix();
  glLoadIdentity();
  glOrtho(0.0, 800.0, 0.0, 600.0, -1, 1);

  glMatrixMode(GL_MODELVIEW);
  glLoadIdentity();

  glTranslatef(10, 590 - 16, 0.0);

  ftglRenderFont(font, "Here I should print some useful information.", FTGL_RENDER_ALL);

  printf("Render");

  glMatrixMode(GL_PROJECTION);
  glPopMatrix();

  glMatrixMode(GL_MODELVIEW);
} */

typedef int (*main_t)(int, char **, char **);
static main_t real_main;

// Rust overrides these
void premain_plugin()
{
  if (DEBUG)
  {
    printf("Glue: Oops, premain not override. \n");
  }
  // die?
  if (getenv("IGNORE_INIT_ERRORS") == NULL)
  {
    exit(37);
  }
}

void postmain_plugin()
{
}
// end rust override

void premain_debug()
{
  printf("Process id: %i, group: %i user: %i, parent pid: %i\n", getpid(), getgid(), getuid(), getppid());
}

int wrap_main(int argc, char **argv, char **envp)
{
  prepare();
  if (DEBUG)
  {
    printf("Glue: Pre-main\n");
    premain_debug();
  }
  premain_plugin();
  int main_res = real_main(argc, argv, envp);
  if (DEBUG)
    printf("Glue: Post-main\n");
  return main_res;
}

// wrap __libc_start_main: replace real_main with wrap_main
int __libc_start_main(
    main_t main, int argc, char **argv,
    main_t init,
    void (*fini)(void), void (*rtld_fini)(void), void *stack_end)
{
  static int (*real___libc_start_main)() = NULL;
  if (!real___libc_start_main)
  {
    if (DEBUG)
      printf("real___libc_start_main = %p (empty)\n", real___libc_start_main);
    char *error;
    real___libc_start_main = dlsym(RTLD_NEXT, "__libc_start_main");
    if (DEBUG)
      printf("real___libc_start_main = %p\n", real___libc_start_main);
    if ((error = dlerror()) != NULL)
    {
      printf("%s\n", error);
      exit(1);
    }
  }
  real_main = main;
  return real___libc_start_main(wrap_main, argc, argv, init, fini, rtld_fini, stack_end);
}