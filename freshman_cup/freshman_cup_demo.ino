#include <RPLidarC1.h>
#include <ESP32Servo.h>
#include "MotorDriver.h"

const int SERVO_PIN = 3;          // 舵机引脚
const int LEFT_MOTOR_PIN1 = 8;    // 左电机引脚1
const int LEFT_MOTOR_PIN2 = 9;    // 左电机引脚2
const int RIGHT_MOTOR_PIN1 = 10;  // 右电机引脚1
const int RIGHT_MOTOR_PIN2 = 11;  // 右电机引脚2

RPLidar lidar;                                                                            // 创建激光雷达对象
Servo servo;                                                                              // 创建舵机对象
MotorDriver motor(LEFT_MOTOR_PIN1, LEFT_MOTOR_PIN2, RIGHT_MOTOR_PIN1, RIGHT_MOTOR_PIN2);  // 创建电机驱动器对象

int angle = 90;
int angle_offset = 5;  // 用于调整舵机中点
int left_speed = 0;
int right_speed = 0;

float distances[360] = { 0 };

void setup() {
  // Serial.begin(115200);
  lidar.begin(Serial2);
  lidar.startScan();
  // 开启计时器，用于舵机PWM控制
  ESP32PWM::allocateTimer(0);
  ESP32PWM::allocateTimer(1);
  ESP32PWM::allocateTimer(2);
  ESP32PWM::allocateTimer(3);
  servo.setPeriodHertz(50);  // 设置舵机PWM频率
  servo.attach(SERVO_PIN);   // 连接舵机引脚
  servo.write(angle + angle_offset);

  motor.begin();  // 开启电机驱动
}

void loop() {
  if (IS_OK(lidar.waitPoint())) {                              // 等到一个新的扫描点
    float distance = lidar.getCurrentPoint().distance / 1000;  // 距离值，单位m
    int angle = lidar.getCurrentPoint().angle;                 // 角度值（整数，四舍五入）
    bool startBit = lidar.getCurrentPoint().startBit;
    byte quality = lidar.getCurrentPoint().quality;

    if (angle >= 0 && angle <= 359) {  // 角度值在[0, 359]范围内
      distances[359 - angle] = distance;
    }

    if (startBit) {     // 每进入一次新的扫描处理并控制一次
      int N = 360 / 8;  // 将360度分成8份
      /*
      1  0  7
      2     6
      3  4  5
      */
      float d_min = 10;  // 最近障碍物距离
      int i_min = 0;     // 最近障碍物对应角度
      for (int i = 0; i < 8 * N; i++) {
        if (distances[i] < d_min && distances[i] > 0.05) {
          d_min = distances[i];
          i_min = i;
        }
      }

      if (d_min > 0.5) {  // 如果最近障碍物距离>0.5m（0.5m内无障碍物）
        angle = 90;       // 直行
        left_speed = right_speed = 60;
      } else {  // 最近障碍物在前方
        if (i_min < N || i_min > 7 * N) {
          angle = 90;
          left_speed = right_speed = 0;            // 停止
        } else if (N < i_min && i_min <= 3 * N) {  // 最近障碍物在左侧
          angle = 80;                              // 右转
          left_speed = 40;
          right_speed = 60;
        } else if (5 * N < i_min && i_min <= 7 * N) {  // 最近障碍物在右侧
          angle = 100;                                 // 左转
          left_speed = 60;
          right_speed = 40;
        } else {
          angle = 90;  // 直行
          left_speed = right_speed = 60;
        }
      }
      // Serial.printf("angle: %3d -> distance: %6.4f\r\n", i_min, d_min);

      servo.write(angle + angle_offset);
      motor.driveAllMotor(left_speed, right_speed);
    }
  } else {  // 出现错误，重新启动雷达
    lidar.startScan();
  }
}