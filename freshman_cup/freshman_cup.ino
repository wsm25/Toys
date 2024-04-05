#include <ESP32Servo.h>
#include <RPLidarC1.h>

#include "MotorDriver.h"
#include "consts.h"
#include "cup_maths.h"

RPLidar lidar; 
Servo servo; 
MotorDriver motor(LEFT_DIR_PIN, LEFT_MOTOR_PIN, 
                  RIGHT_DIR_PIN, RIGHT_MOTOR_PIN);

void setup() {
  #ifdef Debug
  Serial.begin(115200);
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
  #ifdef Debug
  Serial.println("=============================\r\nNew loop begins!");
  #endif
  float rleft=0, rright=0;
  bool oldscan=true;
  do{
    if (IS_FAIL(lidar.waitPoint(lidartimeout))){ // fail, restart and rescan
      lidar.startScan(false, lidartimeout);
      #ifdef Debug
      Serial.println("Lidar off! restarting...");
      #endif
      continue;
    }
    auto&p=lidar.getCurrentPoint(); // include 3 32-bit copy
    oldscan=!p.startBit;
    if (p.distance<min_dist || p.distance>max_dist) continue; // ignore invalid distance
    // get biggest r
    // TODO: crash detection
    if (p.angle>180){ // left
      if (p.angle<min_langle || p.angle>max_langle) continue;
      if (rleft<0) continue;
      float r=calc_rl(p.distance, p.angle);
      if (r<0) rleft=r;
      else if(r>rleft) rleft=r;
    } else {
      if (p.angle<min_rangle || p.angle>max_rangle) continue;
      if (rright<0) continue;
      float r=calc_rr(p.distance, p.angle);
      if (r<0) rright=r;
      else if(r>rright) rright=r;
    }
    #ifdef Debug
    // Serial.printf("New data: (%6.4f, %6.4f)\r\n", p.angle, p.distance);
    #endif
  } while(oldscan);
  float angle;
  if(rleft==0) {
    if (rright==0) angle=0;
    else angle=340;
  } else {
    if (rright==0) angle=20;
    else if(rleft>rright) angle=360-calc_angle(rleft)*(rleft-rright)/(rleft+rright)*3;
    else angle=calc_angle(rright)*(rright-rleft)/(rleft+rright)*3;
  } 
  
  // if(angle<min_angle || angle>360-min_angle) angle=0;
  Speeds speed=calc_speed(angle);
  #ifdef Debug
  Serial.printf("Operation on this loop: rl=%3f, rr=%3f, angle=%3f, speed=%3f\r\n\r\n", 
    rleft, rright, angle, (speed.left+speed.right)/2);
  #endif
  servo.write((-int(angle)+servo_offset+720)%360);
  motor.driveAllMotor(speed.left, speed.right);
}