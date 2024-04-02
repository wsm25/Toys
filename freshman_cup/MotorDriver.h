#ifndef MOTOR_DRIVER_H
#define MOTOR_DRIVER_H

class MotorDriver {
public:
    MotorDriver(int pin1, int pin2, int pin3, int pin4);

    ~MotorDriver();

    void begin(void);

    void driveMotor(int motor, int speed);

    void driveAllMotor(int left_speed, int right_speed);

private:
    int left_dir_pin;
    int left_motor_pin;

    int right_dir_pin;
    int right_motor_pin;

    void driveMotor(int motor_pin, int dir_pin, int speed);
};

#endif