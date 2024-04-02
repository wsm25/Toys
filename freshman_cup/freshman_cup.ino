#include <RPLidarC1.h>
#include <ESP32Servo.h>
#include "MotorDriver.h"

const int SERVO_PIN = 3;  // 舵机引脚

const int LEFT_DIR_PIN = 8;
const int RIGHT_DIR_PIN = 9;
const int LEFT_MOTOR_PIN = 13;
const int RIGHT_MOTOR_PIN = 12;

RPLidar lidar;                                                                    // 创建激光雷达对象
Servo servo;                                                                      // 创建舵机对象
MotorDriver motor(LEFT_DIR_PIN, LEFT_MOTOR_PIN, RIGHT_DIR_PIN, RIGHT_MOTOR_PIN);  // 创建电机驱动器对象

int servo_angle = 90;
int servo_offset = 5;  // 用于调整舵机中点
int left_speed = 0;
int right_speed = 0;

void setup() {
  Serial.begin(115200);
  lidar.begin(Serial2);
  lidar.startScan();
  // 开启计时器，用于舵机PWM控制
  ESP32PWM::allocateTimer(0);
  ESP32PWM::allocateTimer(1);
  ESP32PWM::allocateTimer(2);
  ESP32PWM::allocateTimer(3);
  servo.setPeriodHertz(50);  // 设置舵机PWM频率
  servo.attach(SERVO_PIN);   // 连接舵机引脚
  servo.write(servo_angle + servo_offset);

  motor.begin();  // 开启电机驱动 
  motor.driveAllMotor(left_speed, right_speed);
}

void loop() {
  if (IS_OK(lidar.waitPoint())) {                               // 等到一个新的扫描点
    double distance = lidar.getCurrentPoint().distance / 1000;  // 距离值，单位m
    int angle = lidar.getCurrentPoint().angle;                  // 角度值（整数，四舍五入）
    bool startBit = lidar.getCurrentPoint().startBit;
    byte quality = lidar.getCurrentPoint().quality;

    if (startBit) {  // 每进入一次新的扫描处理并控制一次

      servo.write(servo_angle + servo_offset);
      motor.driveAllMotor(left_speed, right_speed);
    }
  } else {  // 出现错误，重新启动雷达
    lidar.startScan();
  }
}