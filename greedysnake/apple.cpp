#include "apple.h"
#include "wSDL.h"
#include <iostream>

Apple::Apple( const int *color){
    this->color=color;
}

void Apple::generate(){
    this->posx=(rand()%WIDTH)*SQ;
    this->posy=(rand()%HEIGHT)*SQ;
    SnakeNode* _nd=nullptr;
    for(int i=0;i<snakeNum;++i){
        if(snakes[i]->dead) continue; 
        _nd=snakes[i]->head;
        do{
            _nd=_nd->next;
            if(_nd->x==this->posx && _nd->y==this->posy){
                this->generate();
                goto outAppleGenerate;
            }
        }while(_nd->next!=nullptr);
    }
    drawSquare(this->posx,this->posy,this->color);
    outAppleGenerate:;
}

void Apple::print(){
    std::cout<<"apple position: ("<<\
        this->posx<<","<<this->posy<<")"<<std::endl;
}

Apple apple=Apple(DARK_RED);