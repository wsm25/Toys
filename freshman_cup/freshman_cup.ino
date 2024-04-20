#include <ESP32Servo.h>
#include <RPLidarC1.h>

#include "MotorDriver.h"
#include "consts.h"
#include "gogogo.h"
#include "ESP32TimerInterrupt.h"

RPLidar lidar;
Servo servo;
MotorDriver motor;

extern float dist[360];
extern bool valid[360];
volatile bool tick=false;

bool IRAM_ATTR TimerHandler(void * timerNo){
    tick=true;
    return true;
}

void setup() {
    #ifdef Debug
    Serial.begin(115200);  // debug serial
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
    // 定时中断
    ESP32Timer ITimer(1);
    ITimer.attachInterruptInterval(
        400000,
        TimerHandler
    );
    memset(valid, 0, 360*sizeof(bool));
}

void loop() {
    if(tick) { // call next
        tick=false;
        auto next_status=next();
        #ifdef Debug
        Serial.printf("operation on this loop: (%3.1f, %3.1f)\r\n",
            next_status.angle, next_status.velocity);
        #endif
        servo.write(next_status.angle+3);
        motor.drive(next_status.velocity);
        memset(valid, 0, 360*sizeof(bool));
    }
    else { // read data
        while (IS_FAIL(lidar.waitPoint())) {  // fail, restart and rescan
            #ifdef Debug
            Serial.println("lidar fail!");
            #endif
            lidar.startScan(false);
        }
        auto &p = lidar.getCurrentPoint();
        // convert to standard polar angle
        int angle = int(p.angle);
        if (angle <= 90) angle = 90-angle; // 0-90
        else angle = 450-angle; // 91-359
        valid[angle]=true;
        if(p.distance>5000 || p.distance<10) {valid[angle]=false;}
        if (valid[angle]) dist[angle] = p.distance;
    }
}