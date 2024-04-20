#include "gogogo.h"
#include "consts.h"
#include <math.h>
#include <Arduino.h>
// two situations:
// - should run in the middle of left and right bound
// - have one bound in front, should turn

/// return x coordinate of polar cord (r, theta)
inline float x_of(float r, float theta) {
    return r*cosf(theta*M_PI/180.f);
}

const float bound_judge=20;

Go next(const float* dist, const bool *valid){
    // left boundary
    int bound_left=150;
    while((bound_left)>=30){
        while ((bound_left)>=30 && !valid[bound_left]) bound_left--;
        int leftside=bound_left;
        bool flag=false;
        float diff=0;
        while((--bound_left)>=30){
            if(valid[bound_left]){
                int angle=leftside-bound_left;
                diff=(dist[leftside]-dist[bound_left])/angle;
                flag=true;
                break;
            }
        }
        if(flag && (diff>bound_judge || diff<-bound_judge)) break; // boundary found
    }
    // boundary not found, turn!
    if (bound_left<30){
        float sum_left=0, sum_right=0;
        int count_left=0, count_right=0;
        for(int i=30; i<150; i++){
            if (!valid[i]) continue;
            int rightside=i;
            float dx=0;
            while((++rightside)<150){
                if(valid[rightside]){
                    int angle=i-rightside;
                    dx=dist[rightside]-dist[i];
                    break;
                }
            }
            if(dx>0) {sum_left+=(rightside-i)/dist[i]; count_left+=rightside-i;}
            else if(dx<0) {sum_right+=(rightside-i)/dist[i]; count_right+=rightside-i;}
        }
        if(count_left!=0) sum_left/=count_left; // greater is nearer
        if(count_right!=0)sum_right/=count_right;
        float angle=constrain(15000.*(sum_right-sum_left), -10, 10); // TODO
        #ifdef Debug
        Serial.printf("no boundary, turn! sum:(%f, %f), angle: %f\r\n", sum_left, sum_right, angle);
        #endif
        // correction on speed and angle, based on distance
        // from 0 to -angle.
        int to=int(-angle*0.66)+90, count=0;
        float sum=0;
        if(angle<0) for(int i=90; i<to; i++) 
            if(valid[i]) {sum+=dist[i]; count++;}
        else for(int i=90; i>to; i--)
            if(valid[i]) {sum+=dist[i]; count++;}
        /// 100mm as danger zone
        if(count>0){
            sum/=count;
            return Go{speed*constrain(sum*.01f, 0.5f, 1.f), angle*10.f/sqrtf(sum)+90};
        } else return Go{speed*0.5f, angle+90}; // ?
    }
    #ifdef Debug
    Serial.println("boundary found!");
    #endif
    // boundary found, find right bound (which MUST exist)
    int bound_right=30;
    while(bound_right<bound_left){
        while ((bound_right<bound_left) && !valid[bound_right]) bound_right++;
        int rightside=bound_right;
        bool flag=false;
        float diff=0;
        while((++bound_right)<=bound_left){
            if(valid[bound_right]){
                int angle=bound_right-rightside;
                diff=(dist[rightside]-dist[bound_left])/angle;
                flag=true;
                break;
            }
        }
        if(flag && (diff>bound_judge || diff<-bound_judge)) break; // boundary found
    }
    // calculate left and right average distance
    float sum_left = 0;
    int cnt_left = 0;
    for (int i = bound_left; i <= 150; i++) {
        if (valid[i]) {
            sum_left += x_of(dist[i],i)-car_left;
            cnt_left++;
        } 
    }
    if (cnt_left) sum_left /= cnt_left;
    float sum_right = 0; 
    int cnt_right = 0;
    for (int i = bound_right; i >= 30; i--) {
        if (valid[i]) {
            sum_right += x_of(dist[i],i)-car_right;
            cnt_right++;
        }
    }
    if (cnt_right) sum_right /= cnt_right;
    #ifdef Debug
    Serial.printf("boundary found (%d, %d)! sum:(%f, %f), angle: %f\r\n", 
        bound_left, bound_right,
        sum_left, sum_right, -0.15f*(sum_left+sum_right)+90);
    #endif
    float angle=constrain(-12.f*(sum_left+sum_right)/min(abs(sum_right),abs(sum_left)), -10, 10);
    // correction on speed and angle, based on distance
        // from 0 to -angle.
        int to=int(-angle*0.66)+90, count=0;
        float sum=0;
        if(angle<0) for(int i=90; i<to; i++) 
            if(valid[i]) {sum+=dist[i]; count++;}
        else for(int i=90; i>to; i--)
            if(valid[i]) {sum+=dist[i]; count++;}
        /// 100mm as danger zone
        if(count>0){
            sum/=count;
            return Go{speed*constrain(sum*.01f, 0.8f, 1.f), angle*200/sum+90};
        } else return Go{speed*0.5f, angle+90}; // ?
    // go
    return Go{speed, +90};
}