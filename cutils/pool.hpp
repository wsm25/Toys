// Copyright: (c) 2024 wsm25

#ifndef WSM_POOL_HPP
#define WSM_POOL_HPP
#include <cstdlib>
#include <new>

/* 
 * Faster vector for manually-drop types, especially built-in types.
 * 
 * UNSAFE: `T` MUST NOT have custom destructor
 * 
 * Benchmark Result (in average, O1 optimization, Linux):
 * `std::vector`: 3.5 ns/op
 * `Vec`: 1.5 ns/op(Linux)
 */
template<typename T>
class Vec{
    T *from, *end, *cur;
public:
    Vec(){
        from=cur=(T*)malloc(sizeof(T)*4);
        end=cur+4;
    }
    ~Vec(){free(from);}
    void push(T x){
        if(cur!=end){ // hot!
            *(cur++)=x; // UB if T has custom destructor
            return;
        }
        size_t size=((size_t)end - (size_t)from), doubled=size*2;
        from=(T*)realloc(from, doubled);
        cur=(T*)((size_t)from + size);
        end=(T*)((size_t)from + doubled);
        *(cur++)=x;
    }
    // SAFETY: must check empty
    T pop(){return *(--cur);}
    bool empty(){return cur==from;}
    // slow
    size_t len(){return cur-from;}
};

/*
 * Pool: allocate in exponentially growing batch, reducing pressure
 * on allocator.
 * 
 * ## Usage
```cpp
Pool<int> pool;
// the same effect as ptr=new int;
int* ptr=pool.get();
// returns ptr to pool, similar to free(ptr)
pool.put(ptr); 
```
 * 
 * For performance consideration, especially for allowing uninitialized
 * types, we will not provide automatic destruction. You may call
 * placement constructor when getting from pool, and call destructor when
 * dropping(put into pool/simply discard)
 * 
 * Benchmark result (in average, Linux and Windows): 
 * - `Pool`: 2 ns per get/put
 * - stdlib: 25 ns per malloc, 7 ns per free
*/
template<typename T>
class Pool{
    class Buf{
        T *from, *end, *cur;
    public:
        Buf(size_t cap){
            from=cur=(T*)malloc(cap*sizeof(T));
            end=from+cap;
        }
        bool full(){return cur==end;}
        // UNSAFE: assume full==false
        T* get(){return (cur++);}
        size_t cap(){return end-from;}
        T* raw(){return from;}
    };

    Buf buf;
    Vec<T*> used; // bufs
    Vec<T*> idle;
public:
    Pool():buf(Buf(4)){}
    ~Pool(){
        while(!used.empty()) free(used.pop());
        free(buf.raw());
    }
    // gets a pointer from pool, equivalent to `malloc`
    T* get(){
        if(!idle.empty()) return idle.pop(); // hot!
        if(!buf.full()) return buf.get(); // hot!
        used.push(buf.raw());
        new(&buf) Buf(buf.cap()*2);
        return buf.get();
    }
    // puts a pointer back to pool, equivalent to `free`
    void put(T* p){idle.push(p);}
};
#endif