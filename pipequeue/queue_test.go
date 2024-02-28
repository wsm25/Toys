package pipequeue_test

import (
	"sync"
	"testing"
	"time"

	"github.com/wsm25/pipequeue"
)

func TestQueue(t *testing.T) {
	q := pipequeue.NewQueue[byte](127)
	if q == nil {
		t.Error("init error")
	}
	var wg sync.WaitGroup

	producer := func() {
		wg.Add(1)
		for i := 0; i < 10086; i++ {
			q.Put(byte(i))
		}
		wg.Done()
	}

	consumer := func() {
		wg.Add(1)
		for i := 0; i < 10086; i++ {
			if q.Get() != byte(i) {
				t.Error("not equal!")
			}
		}
		wg.Done()
	}
	go producer()
	go consumer()
	wg.Wait()
}

func TestKill(t *testing.T) {
	q := pipequeue.NewQueue[byte](127)
	if q == nil {
		t.Error("init error")
	}
	var wg sync.WaitGroup

	producer := func() {
		wg.Add(1)
		for i := 0; i < 256; i++ {
			q.Put(byte(i))
		}
		wg.Done()
	}

	go producer()
	time.Sleep(100 * time.Millisecond)
	q.Get()
	if q.Get() != 1 {
		t.Error()
	}
	q.Kill()
	if q.Get() != 0 {
		t.Error()
	}
	wg.Wait()
}

func BenchmarkQueue(b *testing.B) {
	q := pipequeue.NewQueue[byte](127)
	var wg sync.WaitGroup
	producer := func() {
		for i := 0; i < b.N; i++ {
			q.Put(byte(i))
		}
		wg.Done()
	}

	consumer := func() {
		for i := 0; i < b.N; i++ {
			q.Get()
		}
		wg.Done()
	}
	wg.Add(2)
	go producer()
	go consumer()
	b.ResetTimer()
	wg.Wait()
}

func BenchmarkChan(b *testing.B) {
	q := make(chan byte, 127)
	var wg sync.WaitGroup
	producer := func() {
		for i := 0; i < b.N; i++ {
			q <- byte(i)
		}
		wg.Done()
	}

	consumer := func() {
		for i := 0; i < b.N; i++ {
			<-q
		}
		wg.Done()
	}
	wg.Add(2)
	go producer()
	go consumer()
	b.ResetTimer()
	wg.Wait()
}
