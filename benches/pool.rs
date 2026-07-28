use divan::black_box;

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 500)]
fn empty_dispatch() {
    neoui::pool::pool().run(&|| {
        black_box(0);
    });
}
