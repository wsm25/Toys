#include <SDL2/SDL.h>
#include "wSDL.h"
#include "consts.h"

/* to draw a rect: 
SDL_Rect fillRect = { 250, 250, 10, 10 };
SDL_SetRenderDrawColor( gRenderer,255,0,0,0 );		
SDL_RenderFillRect( gRenderer, &fillRect );
*/

SDL_Window* gWindow;
SDL_Surface* gScreenSurface;
SDL_Renderer* gRenderer;

void drawmap(){
    SDL_Rect fillRect = { 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT };
	SDL_SetRenderDrawColor( gRenderer, BG_COLOR[0],BG_COLOR[1],BG_COLOR[2],0 );		
	SDL_RenderFillRect( gRenderer, &fillRect );
}

void clear(){
	//Destroy window
	SDL_DestroyWindow( gWindow );
	gWindow = NULL;
	//Quit SDL subsystems
	SDL_Quit();
}

void gerror(std::string info){
	printf( "%s\nSDL_Error: %s\n", info.c_str(), SDL_GetError() );
	clear();
	exit(-1);
}

void initSDL(){
	//Initialize SDL
	if( SDL_Init( SDL_INIT_VIDEO ) < 0 ){
		gerror( "SDL could not initialize!" );
	}
	//Create window
	gWindow = SDL_CreateWindow( TITLE, SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, SCREEN_WIDTH, SCREEN_HEIGHT, SDL_WINDOW_SHOWN );
	if( gWindow == NULL ){
		gerror( "Window could not be created!" );
	}
	//Get window surface
	gScreenSurface = SDL_GetWindowSurface( gWindow );
	//Create renderer for window
	gRenderer = SDL_CreateRenderer( gWindow, -1, SDL_RENDERER_ACCELERATED );
	if( gRenderer == NULL )
		gerror( "Renderer could not be created!");
}

void drawSquare(int posx, int posy,const int *color){
	SDL_SetRenderDrawColor( gRenderer, color[0], color[1], color[2], 0 );
    SDL_Rect _rec={posx, posy,SQ,SQ};
    SDL_RenderFillRect( gRenderer , &_rec);
}