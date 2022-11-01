#include <GL/gl.h>
#include <FTGL/ftgl.h>
#include <stdio.h>
#include <dlfcn.h>

static void init () __attribute__((constructor));
void performOverlay();

void (*_glfwSwapBuffers)() = NULL;

void glfwSwapBuffers()
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
}

static void init()
{
  printf("Init");
}

void performOverlay()
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
}
