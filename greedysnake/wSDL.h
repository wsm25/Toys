#ifndef WSDL_H
#define WSDL_H
#include <SDL2/SDL.h>
#include <iostream>
#include "consts.h"

extern SDL_Window* gWindow;
extern SDL_Surface* gScreenSurface;
extern SDL_Renderer* gRenderer;

void drawmap();
void clear();
void gerror(std::string info);
void initSDL();
void drawSquare(int posx, int posy,const int *color);
#endif