#ifndef SNAKE_H
#define SNAKE_H
#include "consts.h"
#include <string>

class SnakeNode{
public:
    int x,y;
    SnakeNode* next; //指向下一节点指针
    SnakeNode(){
        x=y = -1;
        next = nullptr;
    };
};

/* to traverse: (replace `it`)
SnakeNode* _node=nullptr;
_node=it;
do{
    _node=_node->next;
    // do your thing
}while(_nd->next!=nullptr);
*/


class Snake{
public:
    // variables
    SnakeNode* head;
    Directions dir;
    std::string name;
    const int *color;
    int headcolor[3];
    bool dead=false;
    Snake(const int *xpos,const int *ypos,Directions dir,std::string name,const int* color);
    ~Snake();
    bool show();
    int getLength();
    void move();
    void die();
    void print();
};

void initSnakes();
extern Snake* snakes[];
#endif