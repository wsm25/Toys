#ifndef APPLE_H
#define APPLE_H
#include "snake.h"
#include "consts.h"

class Apple{
    const int *color;
public:
    int posx,posy;
    Apple( const int *color);
    void generate();
    void print();
};

extern Apple apple;
#endif