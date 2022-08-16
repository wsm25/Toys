#ifndef CONSTS_H
#define CONSTS_H

// main window
#define WIDTH 54
#define HEIGHT 32
#define BG_COLOR BLACK
#define SQ 20
const int SCREEN_WIDTH= SQ*WIDTH;
const int SCREEN_HEIGHT= SQ*HEIGHT;
const char TITLE[]="Greedy Snake";

// snake
#define READEVENT 1
#define FPS 25
#define HEAD_COLOR_RATE 0.9
#define snakeNum 3
  // -1 stands for the end
const int XPOS_SNAKE1[]={27,28,29,-1};
const int XPOS_SNAKE2[]={30,31,32,-1};
const int XPOS_SNAKE3[]={33,34,35,-1};

const int YPOS_SNAKE1[]={16,16,16,-1};
const int YPOS_SNAKE2[]={16,16,16,-1};
const int YPOS_SNAKE3[]={16,16,16,-1};

// other consts
enum Directions{UP,DOWN,LEFT,RIGHT};

const int WHITE[]={255, 255, 255};
const int BLACK[]={0, 0, 0};
const int GRAY[]={230, 230, 230};
const int DARK_GRAY[]={40, 40, 40};
const int DARK_GREEN[]={0, 155, 0};
const int GREEN[]={0, 255, 0};
const int RED[]={255, 0, 0};
const int DARK_RED[]={145,0,0};
const int BLUE[]={0, 0, 255};
const int DARK_BLUE[]={0,0, 139};
const int LIGHT_BLUE[]={132,112,255};
const int GOLD[]={255,215,0};

#endif