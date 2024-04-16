#include "gogogo.h"
#include "consts.h"
#include <math.h>

// two situations:
// - should run in the middle of left and right bound
// - have one bound in front, should turn

/// return x coordinate of polar cord (r, theta)
inline float x_of(float r, float theta) {
    return r*cosf(theta*M_PI/180.);
}

const float bound_judge=10;

Go next(float* dist){
    // left boundary
    int bound_left=150;
    while((--bound_left)>=30){
        float diff=dist[bound_left+1]-dist[bound_left];
        if(diff>bound_judge || diff<-bound_judge) break; // boundary found
    }
    // boundary not found, turn!
    if (bound_left<30){
        float sum_left=0, sum_right=0;
        for(int i=30; i<150; i+=3){
            float dx=dist[i+3]-dist[i];
            if(dx>0) sum_left+=1./dist[i];
            else sum_right+=1./dist[i];
        }
        float angle=90+5.*(sum_left-sum_right);
        return Go{speed/3, angle};
    }
    // boundary found, find right bound (which MUST exist)
    int bound_right=30;
    while((++bound_right)<=bound_left){
        float diff=dist[bound_right-1]-dist[bound_right];
        if(diff>bound_judge || diff<-bound_judge) break; // boundary found
    }
    // 
    // TODO: O(log(n)) implement proof
    float left_angle;
    bool left_angle_found=false;
    for(int i=150; i>=bound_left; i--){
        if(x_of(dist[i], i)>car_left) { // crash!
            left_angle=i+1;
            left_angle_found=true;
            break;
        }
    }
    float right_angle;
    bool right_angle_found=false;
    for(int i=30; i<=bound_right; i++){
        if(x_of(dist[i], i)<car_right) { // crash!
            right_angle=i-1;
            right_angle_found=true;
            break;
        }
    }
    if (!(left_angle_found || right_angle_found)) // safe
        return Go{speed, 90};
    if (!left_angle_found && right_angle_found) { // right will crash
        return Go{speed/3, 100};
    }
    if (left_angle_found && !right_angle_found) { // right will crash
        return Go{speed/3, 80};
    }
    // will both crash, boom! (?)
    return Go{speed/5, 90+(left_angle-right_angle)/2};
}