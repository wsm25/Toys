// g++ apple.cpp snake.cpp wSDL.cpp main.cpp -lmingw32 -lSDL2main -lSDL2 -o main.exe 
// semi-static: g++ apple.cpp snake.cpp wSDL.cpp main.cpp -lmingw32 -lSDL2main -lSDL2 -Wl,-Bstatic -lgcc -Wl,-Bstatic -lstdc++ -Wl,-Bstatic -lwinpthread -o main.exe 
/* static: g++ -static apple.cpp snake.cpp wSDL.cpp main.cpp -lmingw32 -lSDL2main -lSDL2 -lsetupapi -lwinmm -limm32 -lversion -lole32 -loleaut32 -lgdi32 -o main_s.exe 
*/

#include <iostream>
#include <SDL2/SDL.h>
#include <windows.h>
#include <time.h>
#include "apple.h"
#include "consts.h"
#include "snake.h"
#include "wSDL.h"

void keyEvent(SDL_Keycode sym);

int main( int argc, char* args[] ){
    // init
    SetConsoleOutputCP(65001);//for utf8
    srand(int(time(0)));
    initSDL();
    drawmap();
    initSnakes();
    apple.generate();
    SDL_RenderPresent( gRenderer );

    // main loop
    bool quit = false;
    SDL_Event e;
    while( !quit ){
        while( SDL_PollEvent( &e ) != 0 ){ //Handle events on queue
			//User requests quit
			if( e.type == SDL_QUIT ){
				quit = true;
                std::cout<<u8"\U0001F622"<<"TERMINATED\n";
                for(int i=0;i<snakeNum;++i){
                    if(!snakes[i]->dead)
                        std::cout<<snakes[i]->name<<"'s length: "<<\
                        snakes[i]->getLength()<<"\n";
                }
			}
			//User presses a key
			else if( e.type == SDL_KEYDOWN )
				keyEvent( e.key.keysym.sym );
		}
        // move
        for(int i=0;i<snakeNum;++i){
            if (snakes[i]->dead) continue;
            snakes[i]->move();
        }
        // check end
        if(snakes[0]->dead+snakes[1]->dead+snakes[2]->dead==2){
            quit=true;
            for(int i=0;i<snakeNum;++i){
                if (!snakes[i]->dead)
                    std::cout<<u8"\U0001F389"<<snakes[i]->name<<" wins!"\
                        <<"length: "<<snakes[i]->getLength()<<std::endl;
            }
        }

        SDL_RenderPresent( gRenderer ); //update
        SDL_Delay( int(1000/FPS) );
    }
    return 0;
}

void keyEvent(SDL_Keycode sym){
    switch( sym ){
        case SDLK_UP:   snakes[0]->dir=UP;break;
        case SDLK_DOWN: snakes[0]->dir=DOWN;break;
        case SDLK_LEFT: snakes[0]->dir=LEFT;break;
        case SDLK_RIGHT:snakes[0]->dir=RIGHT;break;
        case SDLK_w:   snakes[1]->dir=UP;break;
        case SDLK_s: snakes[1]->dir=DOWN;break;
        case SDLK_a: snakes[1]->dir=LEFT;break;
        case SDLK_d:snakes[1]->dir=RIGHT;break;
        case SDLK_k:   snakes[2]->dir=UP;break;
        case SDLK_j: snakes[2]->dir=DOWN;break;
        case SDLK_h: snakes[2]->dir=LEFT;break;
        case SDLK_l:snakes[2]->dir=RIGHT;break;
        default:break;
    }
}