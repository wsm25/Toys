#include "snake.h"
#include "apple.h"
#include "wSDL.h"

Snake::Snake(const int *xpos,const int *ypos, Directions dir,std::string name,const int *color){
    this->head=new SnakeNode;
    SnakeNode* _snake=this->head;
    while(*xpos>=0){
        _snake->next=new SnakeNode;
        _snake=_snake->next;
        _snake->x=*(xpos++)*SQ;
        _snake->y=*(ypos++)*SQ;
    }
    this->dir=dir;
    this->name=name;
    this->color=color;
    for(int i=0;i<3;++i){
        this->headcolor[i]=int(HEAD_COLOR_RATE*color[i]);}
    // TODO: update lighter function
}

Snake::~Snake(){
    SnakeNode* _node=this->head,*_t;
    this->head=nullptr;
    do{
        _t=_node->next;
        delete _node;
        _node=_t;
    }while(_node->next!=nullptr);
}

bool Snake::show(){
    SnakeNode* _node=this->head->next;
    SDL_SetRenderDrawColor( gRenderer, this->headcolor[0], this->headcolor[1], this->headcolor[2], 0 );
    SDL_Rect _rec={_node->x,_node->y,SQ,SQ};
    SDL_RenderFillRect( gRenderer , &_rec);
    SDL_SetRenderDrawColor(gRenderer, this->color[0], this->color[1], this->color[2] ,0);
    do{
        _node=_node->next;
        _rec={_node->x,_node->y,SQ,SQ};
        SDL_RenderFillRect( gRenderer , &_rec);
    }while(_node->next!=nullptr);
    return true;
}

int Snake::getLength(){
    SnakeNode* _node=this->head->next;
    int length=0;
    do{
        ++length;
        _node=_node->next;
    }while(_node->next!=nullptr);
    return length;
}

void Snake::die(){
    this->dead=true;
    SnakeNode* _node=this->head->next; // skip the poor head
    SDL_Rect _rec;
    int length=0;
    SDL_SetRenderDrawColor( gRenderer, BG_COLOR[0],BG_COLOR[1],BG_COLOR[2],0 );		
    do{
        ++length;
        _node=_node->next;
        _rec={_node->x,_node->y,SQ,SQ};
        SDL_RenderFillRect( gRenderer , &_rec);
        
    }while(_node->next!=nullptr);
    std::cout<<u8"\U00002620"<<this->name<<" dead! length: "<<length<<std::endl;
}

void Snake::print(){
    SnakeNode* _node=this->head;
    std::cout<<this->name<<" node: ";
    do{
        _node=_node->next;
        std::cout<<"("<<_node->x<<","<<_node->y<<")   ";
    }while (_node->next!=nullptr);
    std::cout<<std::endl;
}


void Snake::move(){
    SnakeNode* _node=new SnakeNode;
    _node->next=this->head;this->head=_node;_node=_node->next;
    #define MOVEIT(m,n) _node->x=(_node->next->x + m + SCREEN_WIDTH) % SCREEN_WIDTH;\
                        _node->y=(_node->next->y + n + SCREEN_HEIGHT) % SCREEN_HEIGHT ;
    switch (this->dir){
        case UP:
            MOVEIT(0,-SQ);break;
        case DOWN:
            MOVEIT(0,SQ);break;
        case LEFT:
            MOVEIT(-SQ,0);break;
        case RIGHT:
            MOVEIT(SQ,0);break;
    }

    // detect apple
    _node=this->head->next;
    if (!(_node->x==apple.posx&&_node->y==apple.posy)){
        while(_node->next->next!=nullptr)_node=_node->next;//to be optimized
        drawSquare(_node->next->x,_node->next->y,BLACK);
        delete (_node->next);
        _node->next=nullptr;
    }else apple.generate();

    // detect kiss
    _node=this->head->next;
    SnakeNode* _nd=nullptr;
    for(int i=0;i<snakeNum;++i){
        // you can bite yourself!
        if(snakes[i]==this || snakes[i]->dead) continue; 
        _nd=snakes[i]->head;
        do{
            _nd=_nd->next;
            if(_nd->x==_node->x && _nd->y==_node->y){
                this->die();
            }
        }while(_nd->next!=nullptr);
    }
    // draw head
    if (!this->dead){
        drawSquare(_node->x,_node->y,this->headcolor);
        drawSquare(_node->next->x,_node->next->y,this->color);
    }
}

Snake* snakes[3];
Snake snake1=Snake(XPOS_SNAKE1,YPOS_SNAKE1,LEFT,"red",RED);
Snake snake2=Snake(XPOS_SNAKE2,YPOS_SNAKE2,LEFT,"blue",BLUE);
Snake snake3=Snake(XPOS_SNAKE3,YPOS_SNAKE3,LEFT,"green",GREEN);
void initSnakes(){
    snakes[0]=&snake1;snakes[1]=&snake2;snakes[2]=&snake3;
    for(int i=0;i<snakeNum;++i)snakes[i]->show();
}