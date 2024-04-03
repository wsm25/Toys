#ifndef CUP_CONSTS_H
#define CUP_CONSTS_H

#define Debug
// #undef Debug
// loop
const unsigned lidartimeout=100; // in ms
const int goal_count=20;
const int passrate=1; // take 1 point per `passrate`

// offsets
const int servo_offset = 5;  // 用于调整舵机中点
const float car_x1=10;
const float car_y1=10;
const float car_x2=10;
const float car_y2=10;
const float carlen=10;

// ranges
const float max_v=10;
const float max_a=10;
const float min_langle=280;
const float max_langle=315;
const float min_rangle=45;
const float max_rangle=80;

const float min_angle=10;

const float min_dist=10;
const float max_dist=6000;


// pins
const int SERVO_PIN = 3;
const int LEFT_DIR_PIN = 8;
const int RIGHT_DIR_PIN = 9;
const int LEFT_MOTOR_PIN = 13;
const int RIGHT_MOTOR_PIN = 12;

#endif // CUP_CONSTS_H