#include "Arduino.h"
#include "MotorDriver.h"

MotorDriver::MotorDriver(int pin1, int pin2, int pin3, int pin4):
    left_dir_pin(pin1), 
    left_motor_pin(pin2), 
    right_dir_pin(pin3), 
    right_motor_pin(pin4)
{}

MotorDriver::~MotorDriver() {}

void MotorDriver::begin()
{
    pinMode(left_dir_pin, OUTPUT);
    pinMode(left_motor_pin, OUTPUT);
    pinMode(right_dir_pin, OUTPUT);
    pinMode(right_motor_pin, OUTPUT);

    driveAllMotor(0, 0);
}

void MotorDriver::driveMotor(int motor, int speed)
{
    int motor_pin, dir_pin;
    switch (motor)
    {
    case 1:
        motor_pin = left_motor_pin;
        dir_pin = left_dir_pin;
        break;
    case 2:
        motor_pin = right_motor_pin;
        dir_pin = right_dir_pin;
        break;
    default:
        return;
    }

    driveMotor(motor_pin, dir_pin, speed);
}

void MotorDriver::driveMotor(int motor_pin, int dir_pin, int speed)
{
    if (speed >= 0)
    {
        digitalWrite(dir_pin, LOW);
    }
    else
    {
        digitalWrite(dir_pin, HIGH);
    }
    analogWrite(motor_pin, constrain(abs(speed), 0, 255));
}

void MotorDriver::driveAllMotor(int left_speed, int right_speed)
{
    driveMotor(1, left_speed);
    driveMotor(2, right_speed);
}
