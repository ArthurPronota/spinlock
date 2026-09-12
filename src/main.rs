use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering} ;
use std::{hint, thread} ;

// Структура для spinlock
struct Spinlock {
    lock_flag: AtomicBool
}

impl Spinlock {
    pub fn new() ->Self {
        Self { lock_flag: AtomicBool::new(false) }
    }

    pub fn lock(&self) {

        while self.lock_flag.compare_exchange(
                false, 
                true, 
                Ordering::Acquire, 
                Ordering::Relaxed
            ).is_err() {
            // Генерирует машинную инструкцию, сообщающую процессору, что выполняется цикл 
            // активного ожидания («spin-lock»).
            hint::spin_loop();
        }
    }

    pub fn unlock(&self) {
        self.lock_flag.store(false, Ordering::Release);
    }
}

// Счётчик
struct Shared (
    std::cell::UnsafeCell<i32>,
) ;

unsafe impl Sync for Shared {}  // реализация Sync для Shared

fn main() {
    let counter = Arc::new(
            (
                Spinlock::new(),
                Shared(std::cell::UnsafeCell::new(0))
            )
        ) ;

    let mut hands = vec![] ;

    for _ in 0..10 {
        let counter_clone = counter.clone() ;
        hands.push(
            thread::spawn(move || {
                let (spin_lock, counter) = &*counter_clone ;

                for _ in 0..1000 {  // 1000 циклов изменений счётчика в потоке
                    spin_lock.lock();   // получение spin lock

                    unsafe {    // установка нового значения сяётчика
                        // Получение изменяемого значения обёрнутого значения.
                        let ptr = counter.0.get() ;
                        *ptr += 1 ; // модификация счётчика
                    }

                    spin_lock.unlock(); // осовобождение spin lock
                }
            })
        );
    }

    // ожидание заверщения работы всех потоков
    for h in hands {
        h.join().unwrap() ;
    }

    // получение конечного значения счётчика
    let (_ , this_count) = &*counter ;
    unsafe {
        println!("Конечное значение счётчика: {}", *this_count.0.get()) ;   // Out: Конечное значение счётчика: 10000
    }

}
