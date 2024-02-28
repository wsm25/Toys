package pipequeue

import "runtime"

type Queue[T any] struct {
	ir, iw uint32 // index read/written
	mask   uint32 // length-1, must be 2^n-1
	arr    []T    // ringbuf
	alive  bool
}

// length must be $2^n-1$
//
// put and take are designed to run on different cores
//
// we use no locks, but wait cache flush passively
func NewQueue[T any](length int) *Queue[T] {
	if !checkLen(length + 1) { // invalid length
		return nil
	}
	return &Queue[T]{
		mask:  uint32(length),
		ir:    0,
		iw:    0,
		arr:   make([]T, length+1),
		alive: true,
	}
}

func (q *Queue[T]) Put(i T) {
	// wait cache flush passively
	for q.alive && (q.iw-q.ir)&q.mask == q.mask {
		runtime.Gosched()
	}
	if !q.alive {
		return
	}
	// not full!
	q.arr[q.iw&q.mask] = i
	q.iw++
}

func (q *Queue[T]) Get() (i T) {
	// wait cache flush passively
	for q.alive && (q.iw-q.ir)&q.mask == 0 {
		runtime.Gosched()
	}
	if !q.alive {
		return
	}
	// not empty!
	i = q.arr[q.ir&q.mask]
	q.ir++
	return
}

// [Bug] may not flush all
func (q *Queue[T]) Kill() {
	if !q.alive {
		return
	}
	q.alive = false
	// wait cache flush passively
	for (q.iw-q.ir)&q.mask != 0 {
		q.ir = q.iw
		runtime.Gosched()
	}
}

// util
func checkLen(n int) bool {
	if n <= 1 {
		return false
	}
	for (n & 1) != 1 {
		n >>= 1
	}
	return n == 1
}
