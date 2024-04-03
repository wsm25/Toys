#ifndef CUP_MATHS_H
#define CUP_MATHS_H

struct Speeds{float left, right;};
Speeds calc_speed(float r); // with given max_a

float calc_rl(float r, float theta); // with given x1, y1

float calc_rr(float r, float theta); // with given x2, y2

float calc_angle(float r);

#endif // CUP_MATHS_H