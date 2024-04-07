#include <ESP32Servo.h>
#include <RPLidarC1.h>

#include "MotorDriver.h"
#include "consts.h"
#include "gogogo.h"

RPLidar lidar; 
Servo servo;
MotorDriver motor;

void setup() {
  #ifdef Debug
  Serial.begin(115200); // debug serial
  #endif
  lidar.begin(Serial2);
  lidar.startScan(lidartimeout);
  // 开启计时器，用于舵机PWM控制
  ESP32PWM::allocateTimer(0);
  ESP32PWM::allocateTimer(1);
  ESP32PWM::allocateTimer(2);
  ESP32PWM::allocateTimer(3);
  servo.setPeriodHertz(50);  // 设置舵机PWM频率
  servo.attach(SERVO_PIN);   // 连接舵机引脚
  servo.write(servo_offset);
  motor.begin();  // 开启电机驱动
}

void loop() {
  float dist[360]; // valid: 0-359
  for(;;){
    // wait data point
    if (IS_FAIL(lidar.waitPoint(lidartimeout))){ // fail, restart and rescan
      lidar.startScan(false, lidartimeout);
      continue;
    }
    auto &p=lidar.getCurrentPoint(); // include 3 32-bit copy
    if(p.startBit) break; // new group of data
    // convert to standard polar angle
    int angle=int(p.angle);
    if (angle<=90) {angle=90-angle;} // 0-89
    else {angle=450-angle;} // 90-359

    dist[angle]=p.distance;
  }
  Go next_status=next(dist);
  #ifdef Debug
  Serial.printf("Operation on this loop: angle=%3f, speed=%3f\r\n\r\n", 
    next_status.angle, next_status.velocity);
  #endif
  servo.write(next_status.angle);
  motor.drive(next_status.velocity);
}