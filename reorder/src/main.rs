use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Barrier;
use std::thread;

const ORDERING: Ordering = Ordering::Relaxed;

fn reorder(
    acquire: &Barrier,
    release: &Barrier,
    write: &AtomicU32,
    read: &AtomicU32,
    reg: &AtomicU32,
) {
    #[allow(unused)]
    use std::arch::x86_64::_mm_mfence;

    loop {
        // wait for the start signal
        acquire.wait();

        // write to write
        write.store(1, ORDERING);

        // barrier to prevent hardware memory reordering
        //unsafe { _mm_mfence(); }

        // read from read, write to reg
        reg.store(read.load(ORDERING), ORDERING);

        // say we're done for this iteration
        release.wait();
    }
}

fn main() {
    let x = AtomicU32::new(1);
    let y = AtomicU32::new(1);
    let r1 = AtomicU32::new(1);
    let r2 = AtomicU32::new(1);
    let acquire = Barrier::new(3);
    let release = Barrier::new(3);

    thread::scope(|s| {
        // first thread
        s.spawn(|| {
            reorder(&acquire, &release, &x, &y, &r1);
        });

        // second thread
        s.spawn(|| {
            reorder(&acquire, &release, &y, &x, &r2);
        });

        for i in 0u64.. {
            // zero x/y each iteration
            x.store(0, ORDERING);
            y.store(0, ORDERING);

            // allow threads to continue
            acquire.wait();

            // wait for both threads to finish
            release.wait();

            // check for memory reordering
            let r11 = r1.load(ORDERING);
            let r22 = r2.load(ORDERING);

            let reordering = (r11 == 0) && (r22 == 0);
            if reordering {
                println!(r"ERROR r1 = {r11}, r2 = {r22}, iteration = {i}");
                panic!();
            } else {
                println!(r"ALL GOOD! r1 = {r11}, r2 = {r22}, iteration = {i}");
            }
        }
    });
}
