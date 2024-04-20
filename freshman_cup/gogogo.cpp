#include "gogogo.h"
#include "consts.h"
#include <math.h>
#include <Arduino.h>
// two situations:
// - should run in the middle of left and right bound
// - have one bound in front, should turn

float dist[360];
bool valid[360];

const float bound_judge=50;

// angle turning right, 0 if all left of car_left
float angle_right(int from, int to){
    float max_r=0;
    for(int i=from; i<=to; i++){
        if (!valid[i]) continue;
        float theta=float(i)*M_PI/180.f;
        float x=dist[i]*cosf(theta)+car_left;
        float y=dist[i]*sinf(theta)-car_front;
        float r=(x*x+y*y)/(2*x);
        if(r>max_r) max_r=r;
    }
    return max_r!=0 ? asinf(carlen/(max_r))/M_PI*180.f : 0;
}

// angle turning left, 0 if all right of car_right
float angle_left(int from, int to){
    float max_r=0;
    for(int i=from; i<=to; i++){
        if (!valid[i]) continue;
        float theta=float(i)*M_PI/180.f;
        float x=dist[i]*cosf(theta)-car_right;
        float y=dist[i]*sinf(theta)-car_front;
        float r=(x*x+y*y)/(-2*x);
        if(r>max_r) max_r=r;
    }
    return max_r!=0 ? asinf(carlen/(max_r))/M_PI*180.f : 0;
}

// average distance
float avg_dist(int from, int to){
    float sum=0;
    int count=0;
    for(int i=from; i<=to; i++){
        if (!valid[i]) continue;
        count++;
        sum+=dist[i];
    }
    return sum/count;
}

Go next(){
    /*
    for(int i=30; i<=150; i++)
        if(valid[i]) Serial.printf("%d:%3.1f ", i, dist[i]);
    Serial.println();
    */
    // left boundary
    int bound_left=150;
    while((bound_left)>=30){
        while (!valid[bound_left]) {bound_left--; continue;}
        int leftside=bound_left;
        float diff=0;
        while((--bound_left)>=30 && !valid[bound_left]);
        if(bound_left<30) break;
        int angle=leftside-bound_left;
        diff=(dist[leftside]-dist[bound_left])/angle;
        if(diff>bound_judge || diff<-bound_judge) break; // boundary found
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
        float angle;
        if(sum_right>sum_left){ // turn left
            angle=angle_left(30, 150);
        } else if (sum_right<sum_left){ // turn right
            angle=-angle_right(30, 150);
        } else angle=0; // ?
        #ifdef Debug
        Serial.printf("no boundary, turn! sum:(%f, %f), angle: %f\r\n", sum_left, sum_right, angle);
        #endif
        return Go{speed*0.5f, constrain(angle, -10.f, 10.f)+90}; // ?
    }
    // boundary found, search right bound (which MUST exist)
    int bound_right=30;
    while(bound_right<bound_left){
        while (!valid[bound_right]) {bound_right++; continue;}
        int rightside=bound_right;
        while((++bound_right)<=bound_left && !valid[bound_right]);
        if(bound_right>=bound_left) break;
        int angle=bound_right-rightside;
        float diff=(dist[rightside]-dist[bound_right])/angle;
        if((diff>bound_judge || diff<-bound_judge)) {
            break; // boundary found
        }
    }
    // calculate left and right average distance
    float aleft=angle_left(30, bound_right);
    float aright=angle_right(bound_left, 150);
    #ifdef Debug
    Serial.printf("boundary found (%d, %d)! angle:(%f, %f)\r\n", 
        bound_left, bound_right,
        aleft, aright);
    #endif
    float angle;

    if (aleft>aright) angle=aleft;
    else if(aleft<aright) angle=-aright;
    else angle=0;
    return Go{speed, constrain(angle,-10,10)+90};
}