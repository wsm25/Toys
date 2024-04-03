#include "cup_maths.h"
#include "consts.h"
#include <cmath>

Speeds calc_speed(float r){
    return Speeds{max_v, max_v};
}

float calc_rl(float r, float theta){
    return r/(cosf((theta-270)/180*M_PI)*2);
}

float calc_rr(float r, float theta){
    return r/(cosf((90-theta)/180*M_PI)*2);
}

float calc_angle(float r){
    return asinf(carlen/(2*r))/M_PI*180;
}