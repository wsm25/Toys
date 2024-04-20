#ifndef CUP_PID_H
#define CUP_PID_H
class PID{
    float k_p, k_i, k_d;
    float last, sum;
public:
    /// @brief 
    /// @param p,i,d: pid algorithm parameters
    PID(float p, float i, float d):
        k_p(p), k_i(i), k_d(d),
        last(0), sum(0){}
    
    float next(float current){
        sum+=current;
        float output=
            k_p * current +
            k_i * sum +
            k_d * (current-last);
        last=current;
        return output;
    }
};
#endif