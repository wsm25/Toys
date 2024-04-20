#include "Simulator.h"
#include <cmath>
Simulator sim;  // 定义仿真器
/*
本例使用PID算法控制小车前进，并将其中error
定义为小车右前方距离减去小车左前方距离
*/
float error=0,last_error=0,derivative=0;//integral=0
const float k_p=2.5,k_d=0.8;//k_i=0

void setup() {
  sim.begin(Serial);  // 打开与仿真环境的串口通信
}

float get_error(){
  float d_r = 0;
  float d_l = 0;
  int count = 0;
  // 计算右侧障碍物平均距离
  for (int angle = 120; angle < 160; angle++) {
    int temp = sim.read_lidar(angle);
    if (temp != 0) {
      count++;
      d_r += temp;
    }
  }
  d_r = (count == 0 ? 1000 : (d_r / (float)count));
  // 计算左侧障碍物平均距离
  count = 0;
  for (int angle = 200; angle < 240; angle++) {
    int temp = sim.read_lidar(angle);
    if (temp != 0) {
      count++;
      d_l += temp;
    }
  }
  d_l = (count == 0 ? 1000 : (d_l / (float)count));
  return d_r-d_l;
}

float get_dis(){
  float d = 0;
  int count = 0;
  for (int angle = 170; angle < 185; angle++) {
    int temp = sim.read_lidar(angle);
    if (temp != 0) {
      count++;
      d += temp;
    }
  }
  d = (count == 0 ? 1000 : (d / (float)count));
  return d;
}

void pid(float error, float derivative){ //用于发送控制指令
  sim.send_command(13*log10(get_dis()), (k_p*error+k_d*derivative));
}

void loop() {
  if (sim.lidar_ready()) {  // 接收到一次完整的雷达数据
    error = get_error();
    //integral += error;
    //if(abs(integral)>MAX_A)integral *= (MAX_A/abs(integral));
    derivative = error - last_error;
    pid(error, derivative);
    last_error = error;
  }
}

/*******************************************************************************
Simulator成员函数说明:
1. void begin(HardwareSerial &serial): 开启串口serial与仿真环境的通信
2. bool lidar_ready(): 当一次完整的雷达数据(360个)接收完成后返回true, 否则返回false
3. int read_lidar(int angle): 返回角度值为angle对应的距离值, angle范围[0, 359]
4. void send_command(int velocity, int angle): 向仿真环境发送运动指令; velocity为
小车, 速度范围[-30, 30], velocity >= 0小车前进，反之后退; angle为小车舵机角度, 范围
[-30, 30], angle > 0小车右转, < 0左转, 仿真环境最高支持10Hz控制频率

基本代码框架:
#include "Simulator.h"
Simulator sim;  # 定义仿真器

void setup() {
  sim.begin(Serial);  # 打开与仿真环境的串口通信
}

void loop() {
  if (sim.lidar_ready()) {  # 接收到一次完整的雷达数据
    // 在此处处理雷达数据并发送运动指令
    // sim.read_lidar(angle);
    // sim.send_command(velocity, angle);
  }
}
******************************************************************************/
